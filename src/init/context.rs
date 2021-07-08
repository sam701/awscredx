use std::path::PathBuf;

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

pub fn config_file() -> PathBuf {
    util::path_to_absolute(config::CONFIG_FILE_PATH)
}

pub fn config_dir() -> PathBuf {
    config_file().parent().unwrap().to_path_buf()
}
