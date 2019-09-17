use std::path::Path;
use std::fmt::{self, Display, Formatter};
use crate::config;
use std::fs::{self, File};
use std::io::prelude::*;
use std::env;
use ansi_term::{Style, Color};

pub fn run() {
    run_jobs();
}

fn run_jobs() -> Result<(), ()> {
    let home = env::var("HOME").expect("HOME is not set");
    let config_path_str = config::CONFIG_FILE_PATH.replace("~", &home);
    let config_path = Path::new(&config_path_str);
    let dir = config_path.parent().unwrap();
    let styles = Styles::new();

    let executor = JobExecutor::new();
    create_config_dir(dir, &executor)?;
    create_config_file(config_path, &executor)?;
    Ok(())
}

struct JobReport {
    text: String,
    status: JobStatus,
}

impl Display for JobReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        let st = match self.status {
            JobStatus::Created => Style::new().fg(Color::Green).bold().paint("created"),
            JobStatus::AlreadyExists => Style::new().fg(Color::Yellow).bold().paint("already exists"),
        };
        let ws = Style::new().fg(Color::White).bold();
        writeln!(f, " {} {}\n   {} {}", ws.paint("-"), self.text, ws.paint("->"), st)
    }
}

enum JobStatus {
    Created,
    AlreadyExists,
}

struct Styles {
    path: Style,
    helpers: Style,
    success: Style,
    already_done: Style,
    failure: Style,
}

impl Styles {
    fn new() -> Self {
        Styles {
            path: Style::new().fg(Color::White).italic(),
            helpers: Style::new().fg(Color::White).bold(),
            success: Style::new().fg(Color::Green).bold(),
            already_done: Style::new().fg(Color::Yellow).bold(),
            failure: Style::new().fg(Color::Red).bold(),
        }
    }

    fn print_job_title(&self, title: &str) {
        println!(" {} {}\n   {} ", self.helpers.paint("-"), title, self.helpers.paint("->"));
    }
}

struct JobExecutor {
    styles: Styles,
}

impl JobExecutor {
    fn new() -> Self {
        JobExecutor {
            styles: Styles::new(),
        }
    }

    fn run<J>(&self, job_title: String, job: J) -> Result<(), ()>
        where J: FnOnce() -> Result<JobStatus, String> {
        unimplemented!()
    }
}

fn create_config_dir(dir: &Path, executor: &JobExecutor) -> Result<(), ()> {
    executor.run(
        format!("Create configuration directory '{}'", executor.styles.path.paint(dir.to_str().unwrap())),
        || if dir.exists() {
            Ok(JobStatus::AlreadyExists)
        } else {
            fs::create_dir_all(dir)
                .map_err(|e| format!("cannot create directory {}: {}", dir.display(), e))?;
            Ok(JobStatus::Created)
        },
    )
}

fn create_config_file(config_path: &Path, executor: &JobExecutor) -> Result<(), ()> {
    executor.run(
        format!("Create configuration file '{}'", executor.styles.path.paint(config_path.to_str().unwrap())),
        || {
            if config_path.exists() {
                Ok(JobStatus::AlreadyExists)
            }else{
                let file = File::create(config_path)
                    .map_err(|e| format!("cannot create configuration file {}: {}", config_path.display(), e))?;
                let content = include_str!("config-template.toml");
                write!(&file, "{}", content)
                    .map_err(|e| format!("cannot write configuration file: {}", e))?;
                Ok(JobStatus::Created)
            }

        },
    )
}

fn print_success(str: String) {
    println!(" {} {} {}", "-", str, "✓");
}