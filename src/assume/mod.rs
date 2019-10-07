use std::{env, process};
use std::path::Path;
use crate::config::Config;
use crate::credentials::CredentialsFile;
use crate::assume::assumer::RoleAssumer;
use rusoto_core::Region;
use ansi_term::{Style, Color};

mod assumer;

pub fn run(profile: &str, config: &Config) {
    match run_raw(profile, config) {
        Ok(_) => {}
        Err(e) => {
            let err_style = Style::new().fg(Color::Red).bold();
            eprintln!("{}: {}", err_style.paint("ERROR"), e);
            process::exit(1);
        }
    }
}

fn run_raw(profile: &str, config: &Config) -> Result<(), String> {
    let cred_file = CredentialsFile::read_default()?;
    let should_check_newer_version = config.check_new_version && cred_file.get_credentials(&config.mfa_profile).is_none();
    let mut assumer = RoleAssumer::new(
        Region::EuCentral1,
        cred_file,
        config,
    );
    assumer.assume(profile)?;
    print_profile(profile);
    if should_check_newer_version {
        check_newer_version()
    }
    Ok(())
}

fn check_newer_version() {
    match crate::version::check_new_version() {
        Ok(Some(pv)) => eprintln!("{}", &pv),
        Ok(None) => {}
        Err(e) => eprintln!("{}: cannot check for newer version: {}",
                            Style::new().fg(Color::Red).bold().paint("ERROR"), e)
    }
}

fn print_profile(profile_name: &str) {
    match env::var_os("SHELL") {
        Some(shell) => {
            let file = Path::new(&shell)
                .file_name().unwrap()
                .to_str().unwrap();
            match file {
                "fish" => println!("set -xg AWS_PROFILE {}", profile_name),
                _ => println!("export AWS_PROFILE={}", profile_name),
            }
        }
        None => println!("export AWS_PROFILE={}", profile_name)
    }
}
