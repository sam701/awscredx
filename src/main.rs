extern crate ansi_term;
extern crate chrono;
extern crate clap;
extern crate custom_error;
extern crate hyper;
extern crate hyper_proxy;
extern crate hyper_tls;
extern crate linked_hash_map;
extern crate reqwest;
extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_sts;
extern crate serde;
extern crate toml;

use ansi_term::{Color, Style};
use chrono::{DateTime, Duration, Local};

use crate::config::Config;
use crate::credentials::CredentialsFile;

mod config;
mod state;
mod credentials;
mod init;
mod assume;
mod version;
mod util;

fn main() {
    const COMMAND_INIT: &str = "init";
    const COMMAND_ASSUME: &str = "assume";
    const COMMAND_LIST_PROFILES: &str = "list-profiles";
    const COMMAND_LIST_CREDENTIALS: &str = "list-credentials";
    const COMMAND_VERSION: &str = "version";

    let matches = clap::App::new("awscredx")
        .version(version::VERSION)
        .about(format!(r#"AWS credentials management, a.k.a. role assumption made easy.
Run '{}' to create the configuration file and set up shell scripts."#,
                       Style::new().fg(Color::Yellow).paint("awscredx init")).as_str())
        .subcommand(clap::SubCommand::with_name(COMMAND_ASSUME)
            .about("Prints shell commands to assume the role for a given profile")
            .arg(clap::Arg::with_name("profile-name")
                .required(true)
                .help("Profile name which role to assume")))
        .subcommand(clap::SubCommand::with_name(COMMAND_INIT)
            .about("Initializes local environment"))
        .subcommand(clap::SubCommand::with_name(COMMAND_LIST_PROFILES)
            .about("Lists configured profiles with their role ARNs"))
        .subcommand(clap::SubCommand::with_name(COMMAND_LIST_CREDENTIALS)
            .about("Lists current credentials with their expiration times"))
        .subcommand(clap::SubCommand::with_name(COMMAND_VERSION)
            .about("Shows current version and checks for newer version"))
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    match matches.subcommand() {
        (COMMAND_ASSUME, Some(arg)) => {
            let config = read_config();
            assume::run(arg.value_of("profile-name").unwrap(), &config)
        }
        (COMMAND_INIT, _) => init::run(),
        (COMMAND_LIST_PROFILES, _) => print_profiles(),
        (COMMAND_LIST_CREDENTIALS, _) => print_credentials(),
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
    let max_profile_name = c.profiles
        .keys()
        .map(|x| x.as_ref().len())
        .max()
        .unwrap_or(0);
    let width = max_profile_name + 2;
    println!("{:width$}Main profile", &c.main_profile, width = width);
    println!("{:width$}Main profile MFA session", &c.mfa_profile, width = width);
    for (name, prof) in c.profiles.iter() {
        println!("{:width$}{}", name, &prof.role_arn, width = width);
    }
}

fn print_credentials() {
    match CredentialsFile::read_default() {
        Ok(cred_file) => {
            let max_profile_width = cred_file.get_current_credentials_data()
                .map(|x| x.profile_name.len())
                .max()
                .unwrap_or(0);
            let width = max_profile_width + 2;
            let prof_style = Style::new().fg(Color::White).bold();
            let time_style = Style::new().fg(Color::Yellow);
            for cred in cred_file.get_current_credentials_data() {
                print!("{} expires ",
                       prof_style.paint(format!("{:width$}", cred.profile_name, width = width)),
                );
                match cred.expires_at {
                    Some(time) => {
                        let local_time: DateTime<Local> = (*time).into();
                        println!("at {} in {}",
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
    format!("{}:{}", d.num_hours(), d.num_minutes() % 60)
}