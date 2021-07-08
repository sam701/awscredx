use std::env;
use std::path::{Path, PathBuf};

use crate::{config, util};

pub enum Shell {
    Fish,
    Bash,
    Zsh,
}

impl From<&str> for Shell {
    fn from(s: &str) -> Self {
        match s {
            "fish" => Self::Fish,
            "bash" => Self::Bash,
            "zsh" => Self::Zsh,
            x => panic!("Unsupported shell {}", x),
        }
    }
}

impl AsRef<str> for Shell {
    fn as_ref(&self) -> &str {
        match self {
            Shell::Bash => "bash",
            Shell::Fish => "fish",
            Shell::Zsh => "zsh",
        }
    }
}

pub fn shell_config_script(shell: &Shell) -> PathBuf {
    data_dir().join(shell_script(shell))
}

pub fn config_file() -> PathBuf {
    util::path_to_absolute(config::CONFIG_FILE_PATH)
}

pub fn config_dir() -> PathBuf {
    config_file().parent().unwrap().to_path_buf()
}

pub fn init_line(shell: &Shell) -> String {
    let cmd = format!("{} init {}", super::BINARY_NAME, shell.as_ref());
    match shell {
        Shell::Fish => format!("{} | source", &cmd),
        _ => format!("eval $({})", &cmd),
    }
}

pub fn data_dir() -> PathBuf {
    let home_dir = env::var("HOME").expect("HOME is not set");
    Path::new(&home_dir).join(".local/share/awscredx")
}

fn shell_script(shell: &Shell) -> &str {
    match shell {
        Shell::Fish => "script.fish",
        _ => "script.sh",
    }
}

pub fn home_based_path(path: &str) -> String {
    let home = env::var("HOME").expect("HOME is not set");
    path.replace(&home, "$HOME").replace("~", "$HOME")
}
