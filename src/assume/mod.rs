use std::env;
use crate::config;
use std::path::Path;
use crate::config::Config;
use crate::credentials::CredentialsFile;
use crate::assume::assumer::RoleAssumer;
use rusoto_core::Region;

mod assumer;

pub fn run(profile: &str) {
    match run_raw(profile) {
        Ok(_) => {},
        Err(e) => println!("ERROR: {}", e),
    }
}

fn run_raw(profile: &str) -> Result<(), String> {
    match Config::read()? {
        Some(config) => {
            let cred_file = CredentialsFile::read_default()?;
            let mut assumer = RoleAssumer::new(
                Region::EuCentral1,
                cred_file,
                &config,
            );
            assumer.assume(profile)?;
            print_profile(profile);
            Ok(())
        }
        None => {
            Err(format!("configuration file {} does not exist.\nRun 'awscredx init' to initialize your working environment.",
                     config::CONFIG_FILE_PATH))
        }
    }
}

fn print_profile(profile_name: &str) {
    match env::var_os("SHELL") {
        Some(shell) => {
            let file = Path::new(&shell)
                .file_name().unwrap()
                .to_str().unwrap();
            match file {
                "fish" => println!("set -x AWS_PROFILE \"{}\"", profile_name),
                _ => println!("export AWS_PROFILE=\"{}\"", profile_name),
            }
        },
        None => println!("export AWS_PROFILE=\"{}\"", profile_name)
    }
}
