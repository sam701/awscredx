use std::env;

use ansi_term::{Color, Style};
use chrono::{DateTime, Duration, Local, Utc};

use crate::config::Config;
use crate::credentials::CredentialsFile;

mod assume;
mod config;
mod credentials;
mod init;
mod state;
mod util;
mod version;
mod web_console;

fn main() {
    // blabla 5

    const COMMAND_INIT: &str = "init";
    const COMMAND_ASSUME: &str = "assume";
    const COMMAND_LIST_PROFILES: &str = "list-profiles";
    const COMMAND_LIST_CREDENTIALS: &str = "list-credentials";
    const COMMAND_PRINT_PROMPT: &str = "print-prompt";
    const COMMAND_PRINT_EXPIRATION: &str = "print-expiration";
    const COMMAND_VERSION: &str = "version";
    const COMMAND_WEB_CONSOLE_SIGNIN: &str = "web-console-signin";

    const ARG_PROFILE_NAME: &str = "profile-name";
    const ARG_WEB_CONSOLE_SERVICE: &str = "service";
    const ARG_OPEN_IN_BROWSER: &str = "open-in-browser";

    let matches = clap::App::new("awscredx")
        .version(version::VERSION)
        .about(format!(r#"AWS credentials management, a.k.a. role assumption made easy.
Run '{}' to create the configuration file and set up shell scripts."#,
                       Style::new().fg(Color::Yellow).paint("awscredx init")).as_str())
        .subcommand(clap::SubCommand::with_name(COMMAND_ASSUME)
            .about("Prints shell commands to assume the role for a given profile")
            .arg(clap::Arg::with_name(ARG_PROFILE_NAME)
                .required(true)
                .help("Profile name which role to assume")))
        .subcommand(clap::SubCommand::with_name(COMMAND_INIT)
            .about("Initializes local environment"))
        .subcommand(clap::SubCommand::with_name(COMMAND_LIST_PROFILES)
            .about("Lists configured profiles with their role ARNs"))
        .subcommand(clap::SubCommand::with_name(COMMAND_LIST_CREDENTIALS)
            .about("Lists current credentials with their expiration times"))
        .subcommand(clap::SubCommand::with_name(COMMAND_WEB_CONSOLE_SIGNIN)
            .about("Prints web console URL for the current profile ($AWS_PROFILE)")
            .arg(clap::Arg::with_name(ARG_WEB_CONSOLE_SERVICE)
                .long(ARG_WEB_CONSOLE_SERVICE)
                .help("AWS service name, e.g. ec2, ecs, etc.")
                .required(true)
                .takes_value(true))
            .arg(clap::Arg::with_name(ARG_OPEN_IN_BROWSER)
                .long(ARG_OPEN_IN_BROWSER)
                .help("Does not print the web console sign-in URL but opens the URL in browser")))
        .subcommand(clap::SubCommand::with_name(COMMAND_PRINT_PROMPT)
            .about("Prints prompt part for the current profile ($AWS_PROFILE)"))
        .subcommand(clap::SubCommand::with_name(COMMAND_PRINT_EXPIRATION)
            .about("Prints expiration for the current profile ($AWS_PROFILE)"))
        .subcommand(clap::SubCommand::with_name(COMMAND_VERSION)
            .about("Shows current version and checks for newer version"))
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    match matches.subcommand() {
        (COMMAND_ASSUME, Some(arg)) => {
            let config = read_config();
            assume::run(arg.value_of(ARG_PROFILE_NAME).unwrap(), &config)
        }
        (COMMAND_PRINT_PROMPT, _) => print_prompt(),
        (COMMAND_PRINT_EXPIRATION, _) => print_expiration(),
        (COMMAND_INIT, _) => init::run(),
        (COMMAND_LIST_PROFILES, _) => print_profiles(),
        (COMMAND_LIST_CREDENTIALS, _) => print_credentials(),
        (COMMAND_WEB_CONSOLE_SIGNIN, Some(arg)) => web_console::create_signin_url(
            arg.value_of(ARG_WEB_CONSOLE_SERVICE).unwrap(),
            arg.is_present(ARG_OPEN_IN_BROWSER),
        ),
        (COMMAND_VERSION, _) => version::print_version(),
        _ => unreachable!(),
    }
}

fn read_config() -> Config {
    match Config::read() {
        Ok(Some(config)) => config,
        Ok(None) => {
            println!("configuration file {} does not exist.\nRun 'awscredx init' to initialize your working environment.",
                     config::CONFIG_FILE_PATH);
            ::std::process::exit(1);
        }
        Err(e) => {
            println!("Cannot read config: {}", e);
            ::std::process::exit(2);
        }
    }
}

fn print_profiles() {
    let c = read_config();
    let max_profile_name = c
        .profiles
        .keys()
        .map(|x| x.as_ref().len())
        .max()
        .unwrap_or(0);
    let width = max_profile_name + 2;
    println!("{:width$}Main profile", &c.main_profile, width = width);
    println!(
        "{:width$}Main profile MFA session",
        &c.mfa_profile,
        width = width
    );
    for (name, prof) in c.profiles.iter() {
        println!("{:width$}{}", name, &prof.role_arn, width = width);
    }
}

fn print_credentials() {
    match CredentialsFile::read_default() {
        Ok(cred_file) => {
            let max_profile_width = cred_file
                .get_current_credentials_data()
                .map(|x| x.profile_name.len())
                .max()
                .unwrap_or(0);
            let width = max_profile_width + 2;
            let prof_style = Style::new().fg(Color::White).bold();
            let time_style = Style::new().fg(Color::Yellow);
            for cred in cred_file.get_current_credentials_data() {
                print!(
                    "{} expires ",
                    prof_style.paint(format!("{:width$}", cred.profile_name, width = width)),
                );
                match cred.expires_at {
                    Some(time) => {
                        let local_time: DateTime<Local> = (*time).into();
                        println!(
                            "at {} in {}",
                            time_style.paint(local_time.format("%H:%M").to_string()),
                            time_style.paint(format_duration(local_time - Local::now()))
                        );
                    }
                    None => println!("{}", time_style.paint("never")),
                }
            }
        }
        Err(e) => {
            println!("Cannot read credentials file: {}", e);
            ::std::process::exit(3);
        }
    }
}

fn format_duration(d: Duration) -> String {
    format!("{}:{:02}", d.num_hours(), d.num_minutes() % 60)
}

fn print_prompt() {
    if let Ok(profile) = env::var("AWS_PROFILE") {
        match credentials::CredentialExpirations::get(&profile) {
            Ok(Some(ex)) => {
                let duration = ex - Utc::now();
                let expiration_style = if duration > Duration::minutes(10) {
                    Style::new().fg(Color::Green)
                } else {
                    Style::new().fg(Color::Yellow).bold()
                };
                if duration > Duration::zero() {
                    print!(
                        "[{} {}]",
                        Style::new()
                            .fg(Color::White)
                            .bold()
                            .paint(profile)
                            .to_string(),
                        expiration_style
                            .paint(format_duration(duration))
                            .to_string(),
                    )
                }
            }
            Ok(None) => print!(
                "[{} {}]",
                Style::new()
                    .fg(Color::White)
                    .bold()
                    .paint(profile)
                    .to_string(),
                Style::new()
                    .fg(Color::Red)
                    .bold()
                    .paint("expired")
                    .to_string(),
            ),
            Err(e) => eprintln!("ERROR: {}", e),
        }
    }
}

fn print_expiration() {
    if let Ok(profile) = env::var("AWS_PROFILE") {
        match credentials::CredentialExpirations::get(&profile) {
            Ok(Some(ex)) => {
                let duration = ex - Utc::now();
                let expiration_style = if duration > Duration::minutes(10) {
                    Style::new().fg(Color::Green)
                } else {
                    Style::new().fg(Color::Yellow).bold()
                };
                if duration > Duration::zero() {
                    print!(
                        "{}",
                        expiration_style
                            .paint(format_duration(duration))
                            .to_string()
                    )
                }
            }
            Ok(None) => print!(
                "{}",
                Style::new()
                    .fg(Color::Red)
                    .bold()
                    .paint("expired")
                    .to_string()
            ),
            Err(e) => eprintln!("ERROR: {}", e),
        }
    }
}
