use std::collections::HashMap;
use std::env;
use std::fs;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub main_profile: String,
    pub mfa_profile: String,
    pub roles: HashMap<String, String>,
}

const CONFIG_FILE_PATH: &str = "~/.config/awscredx/config.toml";

fn read_raw(path: &str) -> Result<Option<Config>, String> {
    let content = match fs::read_to_string(&path) {
        Ok(c) => c,
        _ => return Ok(None),
    };
    let config: Config = toml::from_str(&content).map_err(|e| format!("Cannot parse TOML file {}: {}", &path, e))?;
    Ok(Some(config))
}

pub fn read() -> Result<Option<Config>, String> {
    let home = env::var("HOME").expect("HOME is not set");
    let path = CONFIG_FILE_PATH.replace("~", &home);
    read_raw(&path)
}

#[test]
fn parse_config() {
    const test_config_path: &str = "./test.config";

    fs::write(test_config_path, r#"
    main_profile = 'abc'
    mfa_profile = 'mfa2'
    [roles]
    role1 = 'yahoo'
    role2 = 'bla'"#).unwrap();

    let cfg = read_raw(test_config_path).unwrap().unwrap();
    println!("cfg = {:?}", &cfg);

    assert_eq!(cfg.main_profile, "abc");
    assert_eq!(cfg.mfa_profile, "mfa2");
    assert_eq!(cfg.roles["role1"], "yahoo");
    assert_eq!(cfg.roles["role2"], "bla");

    fs::remove_file(test_config_path).unwrap();
}