use std::{env, process};
use std::path::Path;

use ansi_term::{Color, Style};
use chrono::{Duration, Utc};
use rusoto_core::Region;

use crate::assume::assumer::RoleAssumer;
use crate::config::Config;
use crate::credentials::CredentialsFile;
use crate::state;

mod assumer;

pub fn run(profile: &str, config: &Config) {
    let err_style = Style::new().fg(Color::Red).bold();
    let error = err_style.paint("ERROR");

    if super::init::outdated_script() {
        println!("{}: detected scripts from the previous version. Please run 'awscredx init'.",
                 &error);
        process::exit(50);
    }
    match run_raw(profile, config) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}: {}", &error, e);
            process::exit(1);
        }
    }
}

fn run_raw(profile: &str, config: &Config) -> Result<(), String> {
    let cred_file = CredentialsFile::read_default()?;

    let mut state = state::State::read();
    let mut assumer = RoleAssumer::new(
        Region::EuCentral1, // TODO: read from envvar
        cred_file,
        config,
    );
    assumer.assume(profile)?;
    print_profile(profile);
    if Utc::now() - state.last_version_check_time > Duration::days(config.check_new_version_interval_days as i64) {
        check_newer_version();
        state.last_version_check_time = Utc::now();
        state.save()?;
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
