extern crate clap;
extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_sts;
extern crate toml;
extern crate serde;
extern crate custom_error;
extern crate chrono;
extern crate ansi_term;
extern crate linked_hash_map;
extern crate reqwest;

use crate::config::Config;
use ansi_term::{Color, Style};

mod config;
mod credentials;
mod init;
mod assume;
mod version;

fn main() {
    let matches = clap::App::new("awscredx")
        .version(version::VERSION)
        .about(format!(r#"AWS credentials management, a.k.a. role assumption made easy.
Run '{}' to create the configuration file and set up shell scripts."#,
                       Style::new().fg(Color::Yellow).paint("awscredx init")).as_str())
        .subcommand(clap::SubCommand::with_name("assume")
            .about("Prints shell commands to assume the role for a given profile")
            .arg(clap::Arg::with_name("profile-name")
                .required(true)
                .help("Profile name which role to assume")))
        .subcommand(clap::SubCommand::with_name("init")
            .about("Initializes local environment"))
        .subcommand(clap::SubCommand::with_name("list-profiles")
            .about("Lists configured profiles with their role ARNs"))
        .subcommand(clap::SubCommand::with_name("version")
            .about("Shows current version and checks for newer version"))
        .setting(clap::AppSettings::SubcommandRequiredElseHelp)
        .get_matches();

    match matches.subcommand() {
        ("assume", Some(arg)) => {
            let config = read_config();
            assume::run(arg.value_of("profile-name").unwrap(), &config)
        }
        ("init", _) =>
            init::run(),
        ("list-profiles", _) =>
            print_profiles(),
        ("version", _) =>
            version::print_version(),
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
        .max().unwrap();
    let width = max_profile_name + 2;
    println!("{:width$}Main profile", &c.main_profile, width = width);
    println!("{:width$}Main profile MFA session", &c.mfa_profile, width = width);
    for (name, prof) in c.profiles.iter() {
        println!("{:width$}{}", name, &prof.role_arn, width = width);
    }
}