use std::path::{Path, PathBuf};
use std::fmt::{self, Display, Formatter};
use crate::config;
use std::fs::{self, File};
use std::io::prelude::*;
use std::env;
use crate::init::context::JobContext;

mod context;

pub fn run() {
    let ctx = context::JobContext::new();
    if let Err(e) = run_jobs(&ctx) {
        println!("{}: {}", ctx.styles.failure.paint("ERROR"), e);
    }
}

macro_rules! job {
    ($name:ident, $ctx:ident) => {
        $name($ctx).map(|r| print_report(&r, $ctx))?
    };
}

fn run_jobs(ctx: &JobContext) -> Result<(), String> {
    job!(create_config_dir, ctx);
    job!(create_config_file, ctx);
    job!(copy_shell_scripts, ctx);
    job!(set_up_script_sources, ctx);
    Ok(())
}

enum JobStatus {
    Success,
    WasAlreadyDone,
}

struct JobReport {
    title: String,
    status: JobStatus,
}

fn print_report(report: &JobReport, context: &JobContext) {
    print!(" {} {}\n   {} ",
           context.styles.helpers.paint("-"),
           &report.title,
           context.styles.helpers.paint("→"));
    match report.status {
        JobStatus::Success =>
            println!("{}", context.styles.success.paint("created")),
        JobStatus::WasAlreadyDone =>
            println!("{}", context.styles.already_done.paint("already exists")),
    }
}

fn create_config_dir(ctx: &JobContext) -> Result<JobReport, String> {
    let title = format!("Create configuration directory '{}'",
                        ctx.styles.path.paint(ctx.config_dir.to_str().unwrap()));
    if ctx.config_dir.exists() {
        Ok(JobReport { title, status: JobStatus::WasAlreadyDone })
    } else {
        fs::create_dir_all(&ctx.config_dir)
            .map_err(|e| format!("cannot create directory {}: {}", ctx.config_dir.display(), e))?;
        Ok(JobReport { title, status: JobStatus::Success })
    }
}

fn create_config_file(ctx: &JobContext) -> Result<JobReport, String> {
    let title = format!("Create configuration file '{}'",
                        ctx.styles.path.paint(ctx.config_script.to_str().unwrap()));
    if ctx.config_script.exists() {
        Ok(JobReport { title, status: JobStatus::WasAlreadyDone })
    } else {
        let file = File::create(&ctx.config_script)
            .map_err(|e| format!("cannot create configuration file {}: {}", ctx.config_script.display(), e))?;
        let content = include_str!("config-template.toml");
        write!(&file, "{}", content)
            .map_err(|e| format!("cannot write configuration file: {}", e))?;
        Ok(JobReport { title, status: JobStatus::Success })
    }
}

fn copy_shell_scripts(ctx: &JobContext) -> Result<JobReport, String> {
    let template_content = ctx.shell_script_content();

    let title = format!("Create shell script '{}'",
                        ctx.styles.path.paint(ctx.config_script.to_str().unwrap()));
    let file = File::create(&ctx.config_script)
        .map_err(|e| format!("cannot create configuration file {}: {}", ctx.config_script.display(), e))?;
    write!(&file, "{}", template_content)
        .map_err(|e| format!("cannot write configuration file: {}", e))?;
    Ok(JobReport { title, status: JobStatus::Success })
}

fn set_up_script_bash() -> Result<(), String> {
    unimplemented!()
}

fn set_up_script_fish(config_dir: &str) -> Result<(), String> {
    unimplemented!()
}

fn set_up_script_sources(ctx: &JobContext) -> Result<JobReport, String> {
    let home_based_config_path = context::home_based_path(ctx.config_script.to_str().unwrap());
    let source_line = format!("source {}", &home_based_config_path);

    let must_attach = match fs::read_to_string(&ctx.shell_init_script) {
        Ok(content) => content.lines().find(|line| line.starts_with(&source_line)).is_none(),
        Err(_) => false,
    };

    let title = format!("Add 'source {}' to {}",
                        ctx.styles.path.paint(&home_based_config_path),
                        ctx.styles.path.paint(ctx.shell_init_script.to_str().unwrap()));
    if must_attach {
        let shell_script_parent = ctx.shell_init_script.parent().unwrap();

        if !shell_script_parent.exists() {
            fs::create_dir_all(shell_script_parent)
                .map_err(|e| format!("cannot create directory {}: {}", shell_script_parent.display(), e))?;
        }

        let f = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .append(true)
            .open(&ctx.shell_init_script)
            .map_err(|e| format!("cannot open config file {}: {}", ctx.shell_init_script.display(), e))?;

        writeln!(&f, "{}\n", &source_line)
            .map_err(|e| format!("cannot write into config file {}: {}", ctx.shell_init_script.display(), e))?;

        Ok(JobReport { title, status: JobStatus::Success })
    } else {
        Ok(JobReport { title, status: JobStatus::WasAlreadyDone })
    }
}
