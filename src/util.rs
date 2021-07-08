use std::path::PathBuf;
use std::{env, fs};

use ansi_term::{Color, Style};
use reqwest::Proxy;

pub fn path_to_absolute(path: &str) -> PathBuf {
    let home = env::var("HOME").expect("HOME is not set");
    let abs = path.replace("~", &home).replace("$HOME", &home);
    PathBuf::from(abs)
}

#[cfg(target_family = "unix")]
pub fn set_permissions(path: &PathBuf, mode: u32) {
    use std::fs::Permissions;
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, Permissions::from_mode(mode)).expect("set file permissions");
}

#[cfg(target_family = "windows")]
pub fn set_permissions(_path: &PathBuf, _mode: u32) {}

pub fn get_https_proxy() -> Option<String> {
    std::env::var_os("https_proxy")
        .or_else(|| std::env::var_os("HTTPS_PROXY"))
        .map(|x| x.into_string().expect("https_proxy is utf8"))
}

pub fn get_https_client() -> Result<reqwest::Client, String> {
    let mut builder = reqwest::ClientBuilder::new();
    if let Some(proxy_url) = get_https_proxy() {
        builder = builder.proxy(
            Proxy::all(&proxy_url)
                .map_err(|e| format!("cannot create proxy from URL({}): {}", &proxy_url, e))?,
        );
    }
    builder
        .build()
        .map_err(|e| format!("cannot build http client: {}", e))
}

pub fn styled_error_word() -> String {
    let err_style = Style::new().fg(Color::Red).bold();
    err_style.paint("ERROR").to_string()
}

pub const STORAGE_DIR: &str = "~/.local/share/awscredx";

pub fn create_storage_dir() {
    let dir = path_to_absolute(STORAGE_DIR);
    if !dir.exists() {
        fs::create_dir_all(&dir).expect("dir created");
        set_permissions(&dir, 0o700);
    }
}
