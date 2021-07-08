use std::fs;
use std::fs::File;
use std::io::Write;
use std::process;

use super::context;
use crate::{styles, util};

pub fn run() {
    if let Err(e) = run_jobs() {
        eprintln!("{}: {}", styles::failure().paint("ERROR"), e);
        process::exit(1);
    }
}

fn run_jobs() -> Result<(), String> {
    create_config_dir()?;
    create_config_file()?;

    println!(
        r#"
Make sure that you:
  {} have filled all necessarily properties in your configuration file {},
  {} have the following line in your shell init script, e.g. .bashrc
     - {} if you use bash,
     - {} if you use zsh,
     - {} if you use fish

Now you can open a new terminal and assume a role by calling '{}'."#,
        styles::number().paint("1."),
        styles::path().paint(context::config_file().to_str().unwrap()),
        styles::number().paint("2."),
        styles::path().paint("eval $(awscredx init bash)"),
        styles::path().paint("eval $(awscredx init zsh)"),
        styles::path().paint("awscredx init fish | source"),
        styles::number().paint("assume <profile-from-your-config>")
    );

    Ok(())
}

fn create_config_dir() -> Result<(), String> {
    if context::config_dir().exists() {
        Ok(())
    } else {
        fs::create_dir_all(&context::config_dir()).map_err(|e| {
            format!(
                "cannot create directory {}: {}",
                context::config_dir().display(),
                e
            )
        })?;
        util::set_permissions(&context::config_dir(), 0o700);
        Ok(())
    }
}

fn create_config_file() -> Result<(), String> {
    if context::config_file().exists() {
        Ok(())
    } else {
        let file = File::create(&context::config_file()).map_err(|e| {
            format!(
                "cannot create configuration file {}: {}",
                context::config_file().display(),
                e
            )
        })?;
        let content = include_str!("templates/config.toml");
        write!(&file, "{}", content)
            .map_err(|e| format!("cannot write configuration file: {}", e))?;
        util::set_permissions(&context::config_file(), 0o600);
        Ok(())
    }
}
