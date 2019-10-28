use std::env;
use std::path::PathBuf;


pub fn path_to_absolute(path: &str) -> PathBuf {
    let home = env::var("HOME").expect("HOME is not set");
    let abs = path.replace("~", &home).replace("$HOME", &home);
    PathBuf::from(abs)
}

#[cfg(target_family = "unix")]
pub fn set_permissions(path: &PathBuf, mode: u32) {
    use std::fs::{self, Permissions};
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, Permissions::from_mode(mode)).expect("set file permissions");
}

#[cfg(target_family = "windows")]
pub fn set_permissions(_path: &PathBuf, _mode: u32) {}

pub fn get_https_proxy() -> Option<String> {
    std::env::var_os("https_proxy")
        .or(std::env::var_os("HTTPS_PROXY"))
        .map(|x| x.into_string().expect("https_proxy is utf8"))
}