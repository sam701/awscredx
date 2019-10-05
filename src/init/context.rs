use std::env;
use std::path::{Path, PathBuf};
use crate::config;
use ansi_term::{Style, Color};

pub struct JobContext {
    pub home_dir: String,
    pub shell: String,
    pub shell_init_script: PathBuf,
    pub config_dir: PathBuf,
    pub config_script: PathBuf,

    pub styles: Styles,
}

impl JobContext {
    pub fn new() -> Self {
        let home_dir = env::var("HOME").expect("HOME is not set");
        let shell = current_shell();
        let config_dir = path_to_absolute(config::CONFIG_FILE_PATH).parent().unwrap().to_path_buf();
        let config_script = Path::new(&config_dir).join(shell_script(&shell)).to_path_buf();
        let shell_init_script = shell_init_script_path(&shell);
        Self {
            home_dir,
            shell,
            shell_init_script,
            config_dir,
            config_script,

            styles: Styles::new(),
        }
    }

    pub fn shell_script_content(&self) -> String {
        let tmpl = self.raw_shell_script_template();
        self.replace_template_placeholders(tmpl)
    }

    fn raw_shell_script_template(&self) -> &str {
        match self.shell.as_str() {
            "fish" => include_str!("script.fish"),
            _ => include_str!("script.sh"),
        }
    }

    fn replace_template_placeholders(&self, template: &str) -> String {
        template
            .replace("@bin@", super::BINARY_NAME)
            .replace("@version@", crate::version::VERSION)
    }
}

fn current_shell() -> String {
    let shell_opt = env::var_os("SHELL");
    match shell_opt.as_ref() {
        Some(shell) => {
            let x: Vec<&str> = shell.to_str().unwrap().split("/").collect();
            let x1 = *x.last().unwrap();
            x1.to_owned()
        }
        None => "bash".to_owned()
    }
}

fn shell_init_script_path(shell: &str) -> PathBuf {
    let bash_files = vec![
        "~/.bashrc",
        "~/.bash_profile",
        "~/.bash_login",
        "~/.profile"
    ];
    let mut abs_bash: Vec<PathBuf> = bash_files.iter()
        .map(|x| path_to_absolute(x)).collect();
    let bash_file_index = first_file_that_exists_index(&abs_bash)
        .unwrap_or(3);
    match shell {
        "fish" => path_to_absolute("~/.config/fish/config.fish"),
        "zsh" => path_to_absolute("~/.zshrc"),
        _ => abs_bash.remove(bash_file_index),
    }
}

fn shell_script(shell: &str) -> &str {
    match shell {
        "fish" => "script.fish",
        _ => "script.sh",
    }
}

fn first_file_that_exists_index(paths: &[PathBuf]) -> Option<usize> {
    paths.iter()
        .position(|p| p.exists())
}

fn path_to_absolute(path: &str) -> PathBuf {
    let home = env::var("HOME").expect("HOME is not set");
    let abs = path.replace("~", &home).replace("$HOME", &home);
    PathBuf::from(abs)
}

pub fn home_based_path(path: &str) -> String {
    let home = env::var("HOME").expect("HOME is not set");
    path.replace(&home, "$HOME").replace("~", "$HOME")
}

pub struct Styles {
    pub path: Style,
    pub helpers: Style,
    pub success: Style,
    pub already_done: Style,
    pub failure: Style,
}

impl Styles {
    fn new() -> Self {
        Styles {
            path: Style::new().fg(Color::White).italic().fg(Color::Cyan),
            helpers: Style::new().fg(Color::White).bold(),
            success: Style::new().fg(Color::Green).bold(),
            already_done: Style::new().fg(Color::Yellow),
            failure: Style::new().fg(Color::Red).bold(),
        }
    }
}