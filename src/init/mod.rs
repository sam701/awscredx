use std::fs::{self, File};
use std::io::prelude::*;
use std::{env, process};
use std::process::Command;
use crate::init::context::JobContext;

mod context;

const BINARY_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn run() {
    let ctx = context::JobContext::new();
    if let Err(e) = run_jobs(&ctx) {
        eprintln!("{}: {}", ctx.styles.failure.paint("ERROR"), e);
        process::exit(1);
    }
}

macro_rules! job {
    ($name:ident, $ctx:ident) => {
        $name($ctx).map(|r| print_report(&r, $ctx))?
    };
}

fn run_jobs(ctx: &JobContext) -> Result<(), String> {
    ensure_tool_in_path()?;
    job!(create_config_dir, ctx);
    job!(create_config_file, ctx);
    job!(write_shell_script, ctx);
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

fn ensure_tool_in_path() -> Result<(), String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("which {}", BINARY_NAME))
        .output()
        .expect("failed to run shell");

    if output.status.success() {
        Ok(())
    } else {
        Err(format!("{} is not in your PATH", BINARY_NAME))
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

fn write_shell_script(ctx: &JobContext) -> Result<JobReport, String> {
    let template_content = ctx.shell_script_content();

    let title = format!("Create shell script '{}'",
                        ctx.styles.path.paint(ctx.config_script.to_str().unwrap()));
    if outdated_script() {
        let file = File::create(&ctx.config_script)
            .map_err(|e| format!("cannot create configuration file {}: {}", ctx.config_script.display(), e))?;
        write!(&file, "{}", template_content)
            .map_err(|e| format!("cannot write configuration file: {}", e))?;
        Ok(JobReport { title, status: JobStatus::Success })
    } else {
        Ok(JobReport { title, status: JobStatus::WasAlreadyDone })
    }
}

pub fn outdated_script() -> bool {
    let shell = env::var("SHELL").expect("SHELL is empty");

    let output = Command::new(&shell)
        .arg("-c")
        .arg("echo $AWSCREDX_SCRIPT_VERSION")
        .output()
        .expect("failed to run shell");
    let version = String::from_utf8(output.stdout)
        .expect("sh output is not UTF-8");
    let version_trimmed = version.trim();

    version_trimmed != env!("CARGO_PKG_VERSION")
}

fn set_up_script_sources(ctx: &JobContext) -> Result<JobReport, String> {
    let home_based_config_path = context::home_based_path(ctx.config_script.to_str().unwrap());
    let source_line = format!("source {}", &home_based_config_path);

    let must_attach = match fs::read_to_string(&ctx.shell_init_script) {
        Ok(content) => content.lines().find(|line| line.starts_with(&source_line)).is_none(),
        Err(_) => false,
    };

    let title = format!("Add 'source {}' to {}",
                        &home_based_config_path,
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
