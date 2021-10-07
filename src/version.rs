use std::fmt::{self, Display};

use ansi_term::{Color, Style};
use serde::Deserialize;

use crate::util;

#[derive(Deserialize)]
struct GithubVersion {
    tag_name: String,
    html_url: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

pub struct PublishedVersion {
    tag_name: String,
    html_url: String,
    binary_download_url: String,
}

impl Display for PublishedVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.pad("")?;
        writeln!(
            f,
            "New version {} is available.",
            Style::new().fg(Color::Yellow).bold().paint(&self.tag_name)
        )?;
        f.pad("")?;
        writeln!(f, "Release notes: {}", &self.html_url)?;
        f.pad("")?;
        writeln!(f, "Binary download URL: {}", &self.binary_download_url)
    }
}

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
const TRAVIS_OS_NAME_OPT: Option<&str> = option_env!("TRAVIS_OS_NAME");

pub fn print_version() {
    println!(
        "awscredx {}",
        Style::new().fg(Color::White).bold().paint(VERSION)
    );
    match check_new_version() {
        Ok(Some(pv)) => println!("{:>2}", &pv),
        Ok(None) => {}
        Err(e) => eprintln!(
            "{}: {}",
            Style::new().fg(Color::Red).bold().paint("ERROR"),
            e
        ),
    }
}

fn os_name() -> &'static str {
    TRAVIS_OS_NAME_OPT.unwrap_or("osx")
}

pub fn check_new_version() -> Result<Option<PublishedVersion>, String> {
    let github_version: GithubVersion = util::get_https_client()?
        .get("https://api.github.com/repos/sam701/awscredx/releases/latest")
        .send()
        .map_err(|e| format!("cannot check new version: {}", e))?
        .json()
        .map_err(|e| format!("cannot decode github version response: {}", e))?;

    if github_version.tag_name == crate::version::VERSION {
        Ok(None)
    } else {
        Ok(Some(PublishedVersion {
            tag_name: github_version.tag_name,
            html_url: github_version.html_url,
            binary_download_url: get_download_url(github_version.assets)
                .ok_or(format!("cannot get download URL for OS={}", os_name()))?,
        }))
    }
}

fn get_download_url(assets: Vec<Asset>) -> Option<String> {
    let osn = os_name();
    assets
        .into_iter()
        .find(|a| osn == extract_os(&a.name))
        .map(|a| a.browser_download_url)
}

fn extract_os(file: &str) -> &str {
    let ix = file.rfind('-').expect("has OS suffix");
    &file[ix + 1..file.len() - 4]
}

#[test]
fn extract_os_name() {
    assert_eq!(extract_os("awscredx-windows.zip"), "windows");
}
