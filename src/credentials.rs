use rusoto_credential::AwsCredentials;
use std::fs::File;
use std::{io, fs};
use std::env;
use std::io::prelude::*;
use std::fmt::{Display, Formatter, Error};
use std::path::{Path, PathBuf};
use std::io::BufReader;
use std::collections::HashMap;
use chrono::{Utc, DateTime};
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct CredentialsFile {
    path: PathBuf,
    profiles: Vec<CredentialsProfile>,
}

#[derive(Debug)]
struct CredentialsProfile {
    profile_name: String,
    credentials: AwsCredentials,
}

const ACCESS_KEY_ID: &str = "aws_access_key_id";
const SECRET_ACCESS_KEY: &str = "aws_secret_access_key";
const SESSION_TOKEN: &str = "aws_session_token";

impl Display for CredentialsProfile {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        if self.profile_name.contains(' ') {
            writeln!(f, "[\"{}\"]", &self.profile_name)?;
        } else {
            writeln!(f, "[{}]", &self.profile_name)?;
        }
        writeln!(f, "{} = {}", ACCESS_KEY_ID, self.credentials.aws_access_key_id())?;
        writeln!(f, "{} = {}", SECRET_ACCESS_KEY, self.credentials.aws_secret_access_key())?;
        if let Some(token) = self.credentials.token() {
            writeln!(f, "{} = {}", SESSION_TOKEN, token)?;
        }
        Ok(())
    }
}

impl CredentialsFile {
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self, io::Error> {
        let mut cf = Self {
            profiles: Vec::new(),
            path: path.as_ref().to_owned(),
        };

        let mut file = match File::open(&path) {
            Ok(f) => f,
            _ => return Ok(cf),
        };
        let br = BufReader::new(&file);


        let mut profile_name = None;
        let mut key_id = None;
        let mut secret_key = None;
        let mut token = None;
        for line_result in br.lines() {
            let ext = line_result?;
            let line = ext.trim();
            if let Some(pn) = read_profile_name(line) {
                if key_id.is_some() {
                    cf.profiles.push(CredentialsProfile {
                        profile_name: profile_name.unwrap(),
                        credentials: AwsCredentials::new(
                            key_id.unwrap(),
                            secret_key.unwrap(),
                            token,
                            None,
                        ),
                    });
                }
                profile_name = Some(pn.to_owned());
                key_id = None;
                secret_key = None;
                token = None;
            } else if let Some(prop) = read_property(line) {
                let val = prop.1.to_owned();
                match prop.0 {
                    ACCESS_KEY_ID => key_id = Some(val),
                    SECRET_ACCESS_KEY => secret_key = Some(val),
                    SESSION_TOKEN => token = Some(val),
                    _ => {}
                }
            }
        }
        cf.profiles.push(CredentialsProfile {
            profile_name: profile_name.unwrap(),
            credentials: AwsCredentials::new(
                key_id.unwrap(),
                secret_key.unwrap(),
                token,
                None,
            ),
        });

        Ok(cf)
    }

    pub fn read_default() -> Result<Self, io::Error> {
        let home = env::var("HOME").expect("HOME is unset");
        Self::read(format!("{}/.aws/credentials", &home))
    }

    pub fn set_profile<P: AsRef<str>>(&mut self, name: P, credentials: AwsCredentials) {
        dbg!(&self.profiles);
        self.profiles.retain(|p| &p.profile_name != name.as_ref());
        dbg!(&self.profiles);
        self.profiles.push(CredentialsProfile {
            profile_name: name.as_ref().to_owned(),
            credentials,
        });
    }

    pub fn write(&self) -> io::Result<()> {
        let file = File::create(&self.path)?;
        for profile in &self.profiles {
            writeln!(&file, "{}", profile)?;
        }
        Ok(())
    }

    pub fn get_credentials(&self, profile_name: &str) -> Option<&AwsCredentials> {
        self.profiles.iter()
            .find(|p| p.profile_name == profile_name)
            .map(|p| &p.credentials)
    }

    pub fn get_profile_names(&self) -> Vec<&str> {
        self.profiles.iter()
            .map(|p| p.profile_name.as_str())
            .collect()
    }
}

fn read_profile_name(line: &str) -> Option<&str> {
    if line.chars().nth(0)? == '[' {
        Some(line.trim_matches(|c| "[ ]\"".contains(c)))
    } else {
        None
    }
}

struct Property<'a>(&'a str, &'a str);

fn read_property(line: &str) -> Option<Property> {
    let parts: Vec<&str> = line.split('=').collect();
    if parts.len() == 2 {
        Some(Property(parts[0].trim(), parts[1].trim_matches(|c| c == ' ' || c == '"')))
    } else {
        None
    }
}

#[test]
fn read_credentials() {
    let mut cred_file = CredentialsFile::read("./test").unwrap();
    cred_file.set_profile("test4", AwsCredentials::new("abc2", "cde2", Some("nice".to_owned()), None));
    assert!(cred_file.get_credentials("test4").is_some());
    assert!(cred_file.get_credentials("test5").is_none());
    cred_file.write().unwrap();
    println!("{:?}", &cred_file);
}


#[derive(Serialize, Deserialize)]
struct CredentialExpirations(HashMap<String, DateTime<Utc>>);

impl CredentialExpirations {
    fn read<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            _ => return Ok(Self(HashMap::new())),
        };
        let hm = toml::from_str(&content).map_err(|e| format!("Cannot parse TOML file {}: {}", path.as_ref().display(), e))?;
        Ok(hm)
    }

    fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let content = toml::to_string(self).expect("Cannot encode expirations into TOML");
        fs::write(path, content).map_err(|e| format!("Cannot write file: {}", e))
    }
}
