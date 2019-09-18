extern crate clap;
extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_sts;
extern crate toml;
extern crate serde;
extern crate custom_error;
extern crate chrono;
extern crate ansi_term;

use crate::config::{Config, Profile};

mod config;
mod credentials;
mod init;
mod assume;

fn main() {
    let matches = clap::App::new("awscredx")
        .version("0.1.0")
        .about("AWS credentials management, a.k.a. role assumption made easy")
        .subcommand(clap::SubCommand::with_name("assume")
            .about("Prints shell commands to assume the role for a given profile")
            .arg(clap::Arg::with_name("profile")
                .value_name("profile-name")
                .required(true)
                .help("Profile name which role to assume")))
        .subcommand(clap::SubCommand::with_name("init")
            .about("Initializes local environment"))
        .subcommand(clap::SubCommand::with_name("list-profiles")
            .about("Lists configured profiles with their role ARNs"))
        .get_matches();

    match matches.subcommand() {
        ("assume", Some(arg)) =>
            assume::run(arg.value_of("profile-name").unwrap()),
        ("init", _) =>
            init::run(),
        ("list-profiles", _) =>
            print_profiles(),
        _ => print_first_time_message()
    }
}

fn print_first_time_message() {
    println!(r#"Welcome to awscredx!

It seems you are running this command for the first time.
Call 'awscredx init' to create the configuration file template and setup a shell helper function."#);
}

fn read_config() -> Config {
    match Config::read() {
        Ok(Some(config)) => config,
        Ok(None) => {
            println!("configuration file {} does not exist.\nRun 'awscredx init' to initialize your working environment.",
                        config::CONFIG_FILE_PATH);
            ::std::process::exit(1);
        },
        Err(e) => {
            println!("Cannot read config: {}", e);
            ::std::process::exit(2);
        }
    }
}

fn print_profiles() {
    let c = read_config();
    let mut pairs: Vec<(&str, &Profile)> = c.profiles.iter().map(|(n,p)| (n.as_ref(), p)).collect();
    pairs.sort_by_key(|x| x.0);
    for (name, prof) in pairs {
        println!("{}", name);
        println!("  {}", prof.role_arn);
    }
}