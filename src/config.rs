use std::env;
use std::fs;
use serde::Deserialize;
use crate::credentials::ProfileName;
use std::io::stdin;
use linked_hash_map::LinkedHashMap;

#[cfg_attr(test, derive(Debug))]
pub struct Config {
    main_profile: ProfileName,
    mfa_profile: ProfileName,
    mfa_serial_number: String,
    pub profiles: LinkedHashMap<ProfileName, Profile>,
}

#[derive(Deserialize, Debug)]
#[cfg_attr(test, derive(Eq, PartialEq))]
pub struct Profile {
    pub role_arn: String,
    pub duration_sec: Option<u64>,
    pub parent_profile: Option<ProfileName>,
    pub color: Option<String>,
}

pub enum AssumeSubject {
    Role {
        role_arn: String,
        session_name: String,
    },
    MfaSession {
        serial_number: String,
        token_code: String,
    },
}

pub const CONFIG_FILE_PATH: &str = "~/.config/awscredx/config.toml";


impl Config {
    pub fn read() -> Result<Option<Config>, String> {
        let home = env::var("HOME").expect("HOME is not set");
        let path = CONFIG_FILE_PATH.replace("~", &home);
        Self::read_raw(&path)
    }

    fn read_raw(path: &str) -> Result<Option<Config>, String> {
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            _ => return Ok(None),
        };

        #[derive(Deserialize, Debug)]
        #[serde(untagged)]
        enum ProfileValue {
            Arn(String),
            ProfileConfig(Profile),
        }

        #[derive(Deserialize, Debug)]
        struct RawConfig {
            main_profile: ProfileName,
            mfa_serial_number: String,
            profiles: LinkedHashMap<ProfileName, ProfileValue>,
        }

        let rc: RawConfig = toml::from_str(&content)
            .map_err(|e| format!("Cannot parse TOML file {}: {}", &path, e))?;
        let mfa = format!("{}_mfa", &rc.main_profile);
        let config = Config {
            main_profile: rc.main_profile,
            mfa_serial_number: rc.mfa_serial_number,
            mfa_profile: ProfileName::new(mfa),
            profiles: rc.profiles.into_iter().map(|(name, value)| (name, match value {
                ProfileValue::Arn(role_arn) => Profile {
                    role_arn,
                    parent_profile: None,
                    duration_sec: None,
                    color: None,
                },
                ProfileValue::ProfileConfig(profile) => profile,
            })).collect(),
        };
        Ok(Some(config))
    }

    pub fn parent_profile(&self, profile: &ProfileName) -> Option<&ProfileName> {
        if profile == &self.main_profile || profile == &self.mfa_profile {
            Some(&self.main_profile)
        } else {
            self.profiles.get(profile)
                .map(|x| x.parent_profile.as_ref().unwrap_or(&self.mfa_profile))
        }
    }

    pub fn assume_subject(&self, profile: &ProfileName) -> Result<Option<AssumeSubject>, String> {
        let res = if profile == &self.mfa_profile {
            Some(AssumeSubject::MfaSession {
                serial_number: self.mfa_serial_number.clone(),
                token_code: read_token_code()?,
            })
        } else {
            self.profiles.get(profile)
                .map(|p| AssumeSubject::Role {
                    role_arn: p.role_arn.clone(),
                    session_name: "awscredx".to_owned(),
                })
        };
        Ok(res)
    }
}

fn read_token_code() -> Result<String, String> {
    eprint!("MFA token: ");
    let mut s = String::with_capacity(10);
    stdin().read_line(&mut s).map_err(|e| format!("cannot read MFA token: {}", e))?;
    let trimmed = s.trim_end();
    if trimmed.is_empty() {
        return Err(format!("empty token"));
    } else {
        Ok(trimmed.to_owned())
    }
}

#[test]
fn parse_config() {
    const TEST_CONFIG_PATH: &str = "./test.config";

    fs::write(TEST_CONFIG_PATH, r#"
    main_profile = 'abc'
    mfa_serial_number = 'mfa2'

    [profiles]
    prof1 = 'arn1'
    prof2 = 'arn2'
    [profiles.prof3]
    role_arn = "arn3"
    parent_profile="prof2""#).unwrap();

    let cfg = Config::read_raw(TEST_CONFIG_PATH).unwrap().unwrap();
    println!("cfg = {:?}", &cfg);

    let arn_prof = |x: &str| Profile {
        role_arn: x.to_owned(),
        parent_profile: None,
        duration_sec: None,
        color: None,
    };

    assert_eq!(cfg.main_profile, ProfileName::new("abc"));
    assert_eq!(cfg.mfa_serial_number, "mfa2".to_owned());
    let prof1 = ProfileName::new("prof1".to_owned());
    let prof2 = ProfileName::new("prof2".to_owned());
    let prof3 = ProfileName::new("prof3".to_owned());

    assert_eq!(cfg.profiles[&prof1], arn_prof("arn1"));
    assert_eq!(cfg.profiles[&prof2], arn_prof("arn2"));
    let pr = &cfg.profiles[&prof3];
    assert_eq!(&pr.role_arn, "arn3");
    assert!(pr.duration_sec.is_none());
    assert!(pr.color.is_none());
    let prof2 = ProfileName::new("prof2");
    let real_prof2 = pr.parent_profile.as_ref().unwrap();
    assert_eq!(real_prof2, &prof2);
    fs::remove_file(TEST_CONFIG_PATH).unwrap();
}