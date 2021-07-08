use std::fs;
use std::fs::File;
use std::io::Write;
use std::process;

use super::{context, styles};
use crate::init::outdated_script;
use crate::util;

enum JobStatus {
    Success,
    WasAlreadyDone,
}

struct JobReport {
    title: String,
    status: JobStatus,
}

pub fn run() {
    if let Err(e) = run_jobs() {
        eprintln!("{}: {}", styles::failure().paint("ERROR"), e);
        process::exit(1);
    }
}

fn run_jobs() -> Result<(), String> {
    let report = create_config_dir()?;
    let report2 = create_config_file()?;

    println!("\nNow edit configuration file {},\nthen open a new terminal and assume a role by calling '{}'",
             styles::path().paint(context::config_file().to_str().unwrap()),
             styles::path().paint("assume <profile-from-your-config>"));

    Ok(())
}

fn print_report(report: &JobReport) {
    print!(
        " {} {}\n   {} ",
        styles::helpers().paint("-"),
        &report.title,
        styles::helpers().paint("<E2><86><92>")
    );
    match report.status {
        JobStatus::Success => println!("{}", styles::success().paint("created")),
        JobStatus::WasAlreadyDone => {
            println!("{}", styles::already_done().paint("already exists"))
        }
    }
}

fn create_config_dir() -> Result<JobReport, String> {
    let title = format!(
        "Create configuration directory '{}'",
        styles::path().paint(context::config_dir().to_str().unwrap())
    );
    if context::config_dir().exists() {
        Ok(JobReport {
            title,
            status: JobStatus::WasAlreadyDone,
        })
    } else {
        fs::create_dir_all(&context::config_dir()).map_err(|e| {
            format!(
                "cannot create directory {}: {}",
                context::config_dir().display(),
                e
            )
        })?;
        util::set_permissions(&context::config_dir(), 0o700);
        Ok(JobReport {
            title,
            status: JobStatus::Success,
        })
    }
}

fn create_config_file() -> Result<JobReport, String> {
    let title = format!(
        "Create configuration file '{}'",
        styles::path().paint(context::config_file().to_str().unwrap())
    );
    if context::config_file().exists() {
        Ok(JobReport {
            title,
            status: JobStatus::WasAlreadyDone,
        })
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
        Ok(JobReport {
            title,
            status: JobStatus::Success,
        })
    }
}
