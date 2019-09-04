use rusoto_credential::AwsCredentials;
use std::fs::File;
use std::fs;
use std::env;
use std::io::prelude::*;
use std::fmt::{Display, Formatter, Error};
use std::path::{Path, PathBuf};
use std::io::BufReader;
use std::collections::HashMap;
use chrono::{Utc, DateTime, Duration};
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
    pub fn read<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let mut cf = Self {
            profiles: Vec::new(),
            path: path.as_ref().to_owned(),
        };

        let file = match File::open(&path) {
            Ok(f) => f,
            _ => return Ok(cf),
        };
        let br = BufReader::new(&file);


        let expiraitons = CredentialExpirations::read(cf.expirations_file_path())?;
        let mut props: HashMap<String, String> = HashMap::new();
        let mut profile_name: Option<String> = None;
        for line_result in br.lines() {
            let ext = line_result.expect("Cannot read line in credentials file");
            let line = ext.trim();
            if let Some(pn) = read_profile_name(line) {
                if let Some(prof_name) = profile_name {
                    cf.add_credentials(prof_name, &mut props, &expiraitons)?;
                }
                profile_name = Some(pn.to_owned());
                props.clear();
            } else if let Some(prop) = read_property(line) {
                props.insert(prop.0.to_owned(), prop.1.to_owned());
            }
        }
        if let Some(prof_name) = profile_name {
            cf.add_credentials(prof_name, &mut props, &expiraitons)?;
        }

        Ok(cf)
    }

    fn expirations_file_path(&self) -> String {
        format!("{}.expirations.toml", self.path.display())
    }

    fn add_credentials(&mut self, profile_name: String, props: &mut HashMap<String, String>, expirations: &CredentialExpirations) -> Result<(), String> {
        let exp = expirations.0.get(&profile_name).cloned();
        if let Some(ex) = exp {
            let now = Utc::now();
            if now - ex > Duration::zero() {
                return Ok(());
            }
        }
        let mut get_key = |key: &str| props.remove(key)
            .ok_or(format!("Profile {} does not have property {}", &profile_name, key));
        let key_id = get_key(ACCESS_KEY_ID)?;
        let key_secret = get_key(SECRET_ACCESS_KEY)?;
        let token = get_key(SESSION_TOKEN)?;
        self.profiles.push(CredentialsProfile {
            profile_name,
            credentials: AwsCredentials::new(
                key_id,
                key_secret,
                Some(token),
                exp,
            ),
        });
        Ok(())
    }

    pub fn read_default() -> Result<Self, String> {
        let home = env::var("HOME").expect("HOME is unset");
        Self::read(format!("{}/.aws/credentials", &home))
    }

    pub fn put_credentials<P: AsRef<str>>(&mut self, profile: P, credentials: AwsCredentials) {
        self.profiles.retain(|p| &p.profile_name != profile.as_ref());
        self.profiles.push(CredentialsProfile {
            profile_name: profile.as_ref().to_owned(),
            credentials,
        });
    }

    pub fn write(&self) -> Result<(), String> {
        let file = File::create(&self.path)
            .map_err(|e| format!("Cannot write file {}: {}", &self.path.display(), e))?;
        let mut expiraitons = CredentialExpirations::new();
        for profile in &self.profiles {
            writeln!(&file, "{}", profile).expect("Cannot write credentials profile");
            if let Some(exp) = profile.credentials.expires_at() {
                expiraitons.0.insert(profile.profile_name.clone(), exp.clone());
            }
        }
        expiraitons.write(self.expirations_file_path())
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
    let now = Utc::now();
    cred_file.put_credentials("test1", AwsCredentials::new("abc2", "cde2", Some("nice".to_owned()), Some(now + Duration::minutes(10))));
    cred_file.put_credentials("test2", AwsCredentials::new("k2", "s2", Some("token1".to_owned()), Some(now - Duration::minutes(10))));
    cred_file.put_credentials("test3", AwsCredentials::new("k2", "s2", Some("token1".to_owned()), None));
    cred_file.write().unwrap();

    cred_file = CredentialsFile::read("./test").unwrap();
    assert!(cred_file.get_credentials("test2").is_none());
    assert_eq!(cred_file.get_credentials("test1").unwrap().aws_access_key_id(), "abc2");
    assert_eq!(cred_file.get_credentials("test1").unwrap().aws_secret_access_key(), "cde2");
    let token = Some("nice".to_owned());
    assert_eq!(cred_file.get_credentials("test1").unwrap().token(), &token);
    let exp = Some(now + Duration::minutes(10));
    assert_eq!(cred_file.get_credentials("test1").unwrap().expires_at(), &exp);
    assert!(cred_file.get_credentials("test3").is_some());

    println!("{:?}", &cred_file);

    fs::remove_file(&cred_file.path).unwrap();
    fs::remove_file(cred_file.expirations_file_path()).unwrap();
}


#[derive(Serialize, Deserialize)]
struct CredentialExpirations(HashMap<String, DateTime<Utc>>);

impl CredentialExpirations {
    fn read<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            _ => return Ok(Self(HashMap::new())),
        };
        let hm = toml::from_str(&content)
            .map_err(|e| format!("Cannot parse TOML file {}: {}", path.as_ref().display(), e))?;
        Ok(CredentialExpirations(hm))
    }

    fn new() -> Self {
        Self(HashMap::new())
    }

    fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), String> {
        let content = toml::to_string(&self.0).expect("Cannot encode expirations into TOML");
        fs::write(path, content).map_err(|e| format!("Cannot write file: {}", e))
    }
}
