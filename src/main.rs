extern crate clap;
extern crate rusoto_core;
extern crate rusoto_credential;
extern crate rusoto_sts;
extern crate toml;
extern crate serde;
extern crate custom_error;
extern crate chrono;
extern crate colored;

use crate::config::Config;
use crate::credentials::CredentialsFile;
use crate::assume::RoleAssumer;
use rusoto_core::Region;

mod config;
mod credentials;
mod init;
mod assume;

fn main() {
    let matches = clap::App::new("awscredx")
        .version("0.1.0")
        .about("AWS credentials management, a.k.a. role assumption made easy")
        .arg(clap::Arg::with_name("assume-role")
            .long("assume-role")
            .takes_value(true)
            .value_name("profile-name")
            .help("Prints shell commands to assume the role for a given profile"))
        .arg(clap::Arg::with_name("init")
            .long("init")
            .help("Initializes local environment"))
        .get_matches();

    let result = if let Some(profile) = matches.value_of("assume-role") {
        run_assume(profile)
    } else if matches.is_present("init") {
        init::run()
    } else {
        print_first_time_message()
    };

    match  result {
        Err(e) => print_error(e),
        Ok(_) => {}
    }

}

fn run_assume(profile: &str) -> Result<(), String> {
    match Config::read()? {
        Some(config) => {
            let cred_file = CredentialsFile::read_default()?;
            let mut assumer = RoleAssumer::new(
                Region::EuCentral1,
                cred_file,
                &config,
            );
            assumer.assume(profile)?;
            println!("Success!");
            Ok(())
        }
        None => {
            Err(format!("configuration file {} does not exist.\nRun 'awscredx --init' to initialize your working environment.",
                        config::CONFIG_FILE_PATH))
        }
    }
}

fn print_error(str: String) {
    println!("ERROR: {}", str)
}

fn print_first_time_message() -> Result<(), String>{
    println!(r#"Welcome to awscredx!

It seems you are running this command for the first time.
Call 'awscredx init' to create the configuration file template and setup a shell helper function."#);
    Ok(())
}