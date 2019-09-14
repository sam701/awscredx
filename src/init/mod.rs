use std::path::Path;
use crate::config;
use std::fs::{self, File};
use std::io::prelude::*;
use std::env;
use colored::Colorize;

pub fn run() -> Result<(), String> {
    let home = env::var("HOME").expect("HOME is not set");
    let config_path_str = config::CONFIG_FILE_PATH.replace("~", &home);
    let config_path = Path::new(&config_path_str);
    if config_path.exists() {
        print_success(format!("Configuration file {} already exists", config::CONFIG_FILE_PATH));
    } else {
        let dir = config_path.parent().unwrap();
        if !dir.exists() {
            fs::create_dir_all(dir)
                .map_err(|e| format!("cannot create directory {}: {}", dir.display(), e))?;
            print_success(format!("Created configuration directory {}", dir.display()));
        }
        let content = include_str!("config-template.toml");
        let file = File::create(config_path)
            .map_err(|e| format!("cannot create configuration file {}: {}", &config_path_str, e))?;
        write!(&file, "{}", content)
            .map_err(|e| format!("cannot write configuration file: {}", e))?;
        print_success(format!("Created configuration file {}", &config_path_str));
    }
    Ok(())
}

fn print_success(str: String) {
    println!(" {} {} {}", "-".bright_white().bold(), str, "✓".bright_green());
}