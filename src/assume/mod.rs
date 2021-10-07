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
use crate::init::SHELL_VAR;
use crate::util;
use crate::{state, styles};
use tokio::runtime::{Builder, Runtime};

mod assumer;
mod main_credentials;

pub fn run(profile: &str, config: &Config) {
    let error = util::styled_error_word();
    if outdated_script() {
        print_update_instructions();
        process::exit(5);
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

    if let Some(check_every_days) = config.check_new_version_interval_days {
        if Utc::now() - state.last_version_check_time > Duration::days(check_every_days as i64) {
            check_newer_version();
            state.last_version_check_time = Utc::now();
            state.save()?;
        }
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
    match env::var_os(SHELL_VAR) {
        Some(shell) => {
            let file = Path::new(&shell).file_name().unwrap().to_str().unwrap();
            match file {
                "fish" => print_fish_profile(profile_name, config),
                "zsh" => print_sh_profile(profile_name, config, true),
                _ => print_sh_profile(profile_name, config, false),
            }
        }
        None => print_sh_profile(profile_name, config, false),
    }
}

fn print_fish_profile(profile_name: &str, config: &Config) {
    println!("set -xg AWS_PROFILE {}; ", profile_name);
    if config.modify_shell_prompt {
        println!(
            r#"function fish_prompt
  set -l old_status $status

  __awscredx_prompt

  echo -n "exit $old_status" | .
  _original_fish_prompt
end
"#
        )
    }
}

fn print_sh_profile(profile_name: &str, config: &Config, zsh: bool) {
    println!("export AWS_PROFILE={}; ", profile_name);
    if config.modify_shell_prompt {
        if zsh {
            println!("setopt PROMPT_SUBST");
        }
        println!("PS1='$(__awscredx_prompt) '${{_ORIGINAL_PS1:-}}")
    }
}

fn get_https_connector() -> Result<ProxyConnector<HttpsConnector<HttpConnector>>, String> {
    let connector = HttpsConnector::new();
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

fn outdated_script() -> bool {
    env::var("AWSCREDX_SCRIPT_VERSION").is_ok()
}

fn print_update_instructions() {
    eprintln!(
        r#"
{}
This new version introduces a breaking change in the script initialization.
Please replace the line {} in your init script with
 - {} for bash,
 - {} for zsh,
 - {} for fish,
Then open a new console window and you can assume a role as before with {}
"#,
        styles::number().paint("ATTENTION!!!"),
        styles::path().paint("source ~/.local/share/awscredx/script.sh"),
        styles::path().paint("eval $(awscredx init bash)"),
        styles::path().paint("eval $(awscredx init zsh)"),
        styles::path().paint("awscredx init fish | source"),
        styles::number().paint("assume <profile-name-in-your-config.toml>"),
    );
}

fn create_runtime() -> Runtime {
    Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("cannot build runtime")
}
