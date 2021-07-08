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

fn replace_template_placeholders(template: &str) -> String {
    template
        .replace("@bin@", super::BINARY_NAME)
        .replace("@version@", crate::version::VERSION)
}

pub fn shell_script_content(shell: &Shell) -> String {
    let tmpl = raw_shell_script_template(shell);
    replace_template_placeholders(tmpl)
}

pub fn init_line(shell: &Shell) -> String {
    let cmd = format!("{} init {}", super::BINARY_NAME, shell.as_ref());
    match shell {
        Shell::Fish => format!("{} | source", &cmd),
        _ => format!("eval $({})", &cmd),
    }
}

fn raw_shell_script_template(shell: &Shell) -> &str {
    match shell {
        Shell::Fish => include_str!("templates/script.fish"),
        _ => include_str!("templates/script.sh"),
    }
}

pub fn data_dir() -> PathBuf {
    let home_dir = env::var("HOME").expect("HOME is not set");
    Path::new(&home_dir).join(".local/share/awscredx")
}

pub fn shell_init_script_path(shell: &Shell) -> PathBuf {
    let bash_files = vec![
        "~/.bashrc",
        "~/.bash_profile",
        "~/.bash_login",
        "~/.profile",
    ];
    let mut abs_bash: Vec<PathBuf> = bash_files
        .iter()
        .map(|x| util::path_to_absolute(x))
        .collect();
    let bash_file_index = first_file_that_exists_index(&abs_bash).unwrap_or(0);
    match shell {
        Shell::Fish => util::path_to_absolute("~/.config/fish/config.fish"),
        Shell::Zsh => util::path_to_absolute("~/.zshrc"),
        _ => abs_bash.remove(bash_file_index),
    }
}

fn shell_script(shell: &Shell) -> &str {
    match shell {
        Shell::Fish => "script.fish",
        _ => "script.sh",
    }
}

fn first_file_that_exists_index(paths: &[PathBuf]) -> Option<usize> {
    paths.iter().position(|p| p.exists())
}

pub fn home_based_path(path: &str) -> String {
    let home = env::var("HOME").expect("HOME is not set");
    path.replace(&home, "$HOME").replace("~", "$HOME")
}
