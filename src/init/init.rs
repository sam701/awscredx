use std::env;
use std::fs;

use super::context;

use crate::init::context::Shell;
use crate::init::styles;
use crate::util;
use std::path::Path;

pub const BINARY_NAME: &str = env!("CARGO_PKG_NAME");

pub enum InitType {
    Bootstrap,
    Full,
}

pub fn run(shell: &str, init_type: InitType) {
    if outdated_script() {
        upgrade_unmaintained_version(shell.into());
    }

    print_bootstrap_script(shell, init_type);
}

fn print_bootstrap_script(shell: &str, init_type: InitType) {
    let buf = env::current_exe().unwrap();
    let bin = buf.to_str().unwrap();

    use InitType::*;
    match init_type {
        Bootstrap => {
            let cmd = format!(r#""{bin}" init --full {shell}"#, bin = bin, shell = shell);
            if shell == "fish" {
                print!(r#"source ({cmd} | psub)"#, cmd = cmd);
            } else {
                print!(r#"source <({cmd})"#, cmd = cmd);
            }
        }
        Full => {
            let tmpl = if shell == "fish" {
                include_str!("templates/script.fish")
            } else {
                include_str!("templates/script.sh")
            }
            .replace("@bin@", super::BINARY_NAME);
            println!("{}", tmpl);
        }
    }
}

pub fn outdated_script() -> bool {
    let storage_dir = Path::new(util::STORAGE_DIR);
    storage_dir.join("script.sh").exists() || storage_dir.join("script.fish").exists()
}

/// This upgrades old awscredx version that used an external script file.
fn upgrade_unmaintained_version(shell: Shell) {
    let cfg_script_path = context::shell_config_script(&shell);
    if cfg_script_path.exists() {
        fs::remove_file(&cfg_script_path).expect("file was deleted");
    }

    println!(
        "{} This new version introduces breaking changes in the script bootstrap process",
        styles::helpers().paint("IMPORTANT!!!")
    );
    println!(
        "You should update your shell init script, e.g. {} like this",
        styles::path().paint(cfg_script_path.to_str().unwrap())
    );

    let home_based_config_path = context::home_based_path(cfg_script_path.to_str().unwrap());
    let source_line = format!("source {}", &home_based_config_path);
    println!(" - Remove line '{}'", &source_line);
    println!(" - Add line '{}'", context::init_line(&shell));
}
