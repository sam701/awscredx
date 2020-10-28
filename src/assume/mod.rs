use std::path::Path;
use std::{env, process};

use ansi_term::{Color, Style};
use chrono::{Duration, Utc};
use hyper::client::HttpConnector;
use hyper::Uri;
use hyper_proxy::{Intercept, Proxy, ProxyConnector};
use hyper_tls::HttpsConnector;

use crate::assume::assumer::RoleAssumer;
use crate::config::Config;
use crate::credentials::CredentialsFile;
use crate::state;
use crate::util;

mod assumer;
mod main_credentials;

pub fn run(profile: &str, config: &Config) {
    let error = util::styled_error_word();
    if super::init::outdated_script() {
        println!(
            "{}: detected scripts from the previous version. Please run 'awscredx init'.",
            &error
        );
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
    let mut cred_file = CredentialsFile::read_default()?;
    if cred_file.get_credentials(&config.main_profile).is_none() {
        return Err(format!("You specified main_profile=\"{prof}\" but there is no profile with this name in your credentials file ", prof = &config.main_profile));
    }
    let mut state = state::State::read();

    let mut assumer = RoleAssumer::new(config.region.clone(), &mut cred_file, config);
    assumer.assume(profile)?;
    print_profile(profile, config);
    if Utc::now() - state.last_version_check_time
        > Duration::days(config.check_new_version_interval_days as i64)
    {
        check_newer_version();
        state.last_version_check_time = Utc::now();
        state.save()?;
    }

    main_credentials::rotate_if_needed(config, &mut cred_file, &mut state)?;

    Ok(())
}

fn check_newer_version() {
    match crate::version::check_new_version() {
        Ok(Some(pv)) => eprintln!("{}", &pv),
        Ok(None) => {}
        Err(e) => eprintln!(
            "{}: cannot check for newer version: {}",
            Style::new().fg(Color::Red).bold().paint("ERROR"),
            e
        ),
    }
}

fn print_profile(profile_name: &str, config: &Config) {
    match env::var_os("SHELL") {
        Some(shell) => {
            let file = Path::new(&shell).file_name().unwrap().to_str().unwrap();
            match file {
                "fish" => {
                    print!("set -xg AWS_PROFILE {}; ", profile_name);
                    println!(
                        "set -l __awscredx_modify_prompt {}",
                        config.modify_shell_prompt
                    );
                }
                _ => {
                    print!("export AWS_PROFILE={}; ", profile_name);
                    println!("__awscredx_modify_prompt={}", config.modify_shell_prompt);
                }
            }
        }
        None => println!("export AWS_PROFILE={}", profile_name),
    }
}

fn get_https_connector() -> Result<ProxyConnector<HttpsConnector<HttpConnector>>, String> {
    let connector = HttpsConnector::new(2).expect("connector with 2 threads");
    Ok(match util::get_https_proxy() {
        Some(proxy_url) => {
            let url = proxy_url
                .parse::<Uri>()
                .map_err(|e| format!("cannot parse proxy URL({}): {}", &proxy_url, e))?;
            let proxy = Proxy::new(Intercept::All, url);
            ProxyConnector::from_proxy(connector, proxy).expect("proxy created")
        }
        None => ProxyConnector::new(connector).expect("transparent proxy created"),
    })
}
