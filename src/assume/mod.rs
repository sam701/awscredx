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
        },
    }
}

fn run_raw(profile: &str, config: &Config) -> Result<(), String> {
    let cred_file = CredentialsFile::read_default()?;
    let mut assumer = RoleAssumer::new(
        Region::EuCentral1,
        cred_file,
        config,
    );
    assumer.assume(profile)?;
    print_profile(profile);
    Ok(())
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
