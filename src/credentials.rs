use std::collections::HashMap;
use std::fmt::{Display, Error, Formatter};
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use chrono::{DateTime, Duration, Utc};
use rusoto_credential::AwsCredentials;
use serde::{Deserialize, Serialize};

use crate::util;

#[derive(Debug)]
pub struct CredentialsFile {
    path: PathBuf,
    expirations_path: PathBuf,
    profiles: Vec<CredentialsProfile>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Ord, PartialOrd)]
#[serde(transparent)]
pub struct ProfileName(Rc<String>);

impl ProfileName {
    pub fn new<S: ToString>(name: S) -> Self {
        Self(Rc::new(name.to_string()))
    }
}

impl Display for ProfileName {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        f.pad(&self.0)
    }
}

impl AsRef<str> for ProfileName {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug)]
struct CredentialsProfile {
    profile_name: ProfileName,
    credentials: AwsCredentials,
}

const ACCESS_KEY_ID: &str = "aws_access_key_id";
const SECRET_ACCESS_KEY: &str = "aws_secret_access_key";
const SESSION_TOKEN: &str = "aws_session_token";

impl Display for CredentialsProfile {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        if self.profile_name.as_ref().contains(' ') {
            writeln!(f, "[\"{}\"]", &self.profile_name)?;
        } else {
            writeln!(f, "[{}]", &self.profile_name)?;
        }
        writeln!(
            f,
            "{} = {}",
            ACCESS_KEY_ID,
            self.credentials.aws_access_key_id()
        )?;
        writeln!(
            f,
            "{} = {}",
            SECRET_ACCESS_KEY,
            self.credentials.aws_secret_access_key()
        )?;
        if let Some(token) = self.credentials.token() {
            writeln!(f, "{} = {}", SESSION_TOKEN, token)?;
        }
        Ok(())
    }
}

pub struct CredentialsData<'a> {
    pub profile_name: &'a str,
    pub expires_at: &'a Option<DateTime<Utc>>,
}

impl CredentialsFile {
    pub fn read<P: AsRef<Path>>(path: P, expirations_path: P) -> Result<Self, String> {
        let mut cf = Self {
            profiles: Vec::new(),
            path: path.as_ref().to_owned(),
            expirations_path: expirations_path.as_ref().to_owned(),
        };

        let file = match File::open(&path) {
            Ok(f) => f,
            _ => return Ok(cf),
        };
        let br = BufReader::new(&file);

        let expiraitons = CredentialExpirations::read(&cf.expirations_path)?;
        let mut props: HashMap<String, String> = HashMap::new();
        let mut profile_name: Option<ProfileName> = None;
        for line_result in br.lines() {
            let ext = line_result.expect("Cannot read line in credentials file");
            let line = ext.trim();
            if line.is_empty() {
                continue;
            }
            if let Some(pn) = read_profile_name(line) {
                if let Some(prof_name) = profile_name {
                    cf.add_credentials(prof_name, &mut props, &expiraitons)?;
                }
                profile_name = Some(ProfileName(Rc::new(pn.to_owned())));
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

    fn add_credentials(
        &mut self,
        profile_name: ProfileName,
        props: &mut HashMap<String, String>,
        expirations: &CredentialExpirations,
    ) -> Result<(), String> {
        let mut get_key = |key: &str| {
            props.remove(key).ok_or(format!(
                "Profile {} does not have property {}",
                &profile_name, key
            ))
        };
        let key_id = get_key(ACCESS_KEY_ID)?;
        let key_secret = get_key(SECRET_ACCESS_KEY)?;
        let token = props.remove(SESSION_TOKEN);

        let exp = expirations.0.get(&profile_name).cloned();
        match exp {
            Some(ex) => {
                let now = Utc::now();
                if now - ex > Duration::zero() {
                    return Ok(());
                }
            }
            None => {
                if token.is_some() {
                    return Ok(());
                }
            }
        }

        self.profiles.push(CredentialsProfile {
            profile_name,
            credentials: AwsCredentials::new(key_id, key_secret, token, exp),
        });
        Ok(())
    }

    pub fn read_default() -> Result<Self, String> {
        let cr = util::path_to_absolute("~/.aws/credentials");
        let ex = util::path_to_absolute(EXPIRATIONS_FILE);
        Self::read(cr, ex)
    }

    pub fn put_credentials(&mut self, profile: ProfileName, credentials: AwsCredentials) {
        self.profiles.retain(|p| p.profile_name != profile);
        self.profiles.push(CredentialsProfile {
            profile_name: profile,
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
                expiraitons.0.insert(profile.profile_name.clone(), *exp);
            }
        }
        expiraitons.write(&self.expirations_path)
    }

    pub fn get_credentials(&self, profile_name: &ProfileName) -> Option<&AwsCredentials> {
        self.profiles
            .iter()
            .find(|p| p.profile_name == *profile_name)
            .map(|p| &p.credentials)
    }

    pub fn get_current_credentials_data(&self) -> impl Iterator<Item = CredentialsData> + '_ {
        self.profiles.iter().map(|x| CredentialsData {
            profile_name: &x.profile_name.0,
            expires_at: x.credentials.expires_at(),
        })
    }
}

fn read_profile_name(line: &str) -> Option<&str> {
    if line.chars().next()? == '[' {
        Some(line.trim_matches(|c| "[ ]\"".contains(c)))
    } else {
        None
    }
}

struct Property<'a>(&'a str, &'a str);

fn read_property(line: &str) -> Option<Property> {
    let parts: Vec<&str> = line.splitn(2, '=').collect();
    if parts.len() == 2 {
        Some(Property(
            parts[0].trim(),
            parts[1].trim_matches(|c| c == ' ' || c == '"'),
        ))
    } else {
        None
    }
}

#[test]
fn read_credentials() {
    let mut cred_file = CredentialsFile::read("./test", "./test.expirations.toml").unwrap();
    let now = Utc::now();
    cred_file.put_credentials(
        ProfileName::new("test1"),
        AwsCredentials::new(
            "abc2",
            "cde2",
            Some("nice".to_owned()),
            Some(now + Duration::minutes(10)),
        ),
    );
    cred_file.put_credentials(
        ProfileName::new("test2"),
        AwsCredentials::new(
            "k2",
            "s2",
            Some("token1".to_owned()),
            Some(now - Duration::minutes(10)),
        ),
    );
    cred_file.put_credentials(
        ProfileName::new("test3"),
        AwsCredentials::new("k2", "s2", Some("token1".to_owned()), None),
    );
    cred_file.put_credentials(
        ProfileName::new("test4"),
        AwsCredentials::new(
            "k4",
            "s4",
            Some("token=4".to_owned()),
            Some(now + Duration::minutes(10)),
        ),
    );
    cred_file.write().unwrap();

    cred_file = CredentialsFile::read("./test", "./test.expirations.toml").unwrap();
    let pn_test1 = ProfileName::new("test1");
    let pn_test2 = ProfileName::new("test2");
    let pn_test3 = ProfileName::new("test3");
    let pn_test4 = ProfileName::new("test4");
    assert!(cred_file.get_credentials(&pn_test2).is_none());
    assert_eq!(
        cred_file
            .get_credentials(&pn_test1)
            .unwrap()
            .aws_access_key_id(),
        "abc2"
    );
    assert_eq!(
        cred_file
            .get_credentials(&pn_test1)
            .unwrap()
            .aws_secret_access_key(),
        "cde2"
    );
    let token = Some("nice".to_owned());
    assert_eq!(
        cred_file.get_credentials(&pn_test1).unwrap().token(),
        &token
    );
    let exp = Some(now + Duration::minutes(10));
    assert_eq!(
        cred_file.get_credentials(&pn_test1).unwrap().expires_at(),
        &exp
    );
    assert!(cred_file.get_credentials(&pn_test3).is_none());
    assert_eq!(
        cred_file
            .get_credentials(&pn_test4)
            .unwrap()
            .token()
            .as_ref()
            .unwrap()
            .as_str(),
        "token=4"
    );

    println!("{:?}", &cred_file);

    fs::remove_file(&cred_file.path).unwrap();
    fs::remove_file(&cred_file.expirations_path).unwrap();
}

pub struct CredentialExpirations(HashMap<ProfileName, DateTime<Utc>>);

const EXPIRATIONS_FILE: &str = "~/.local/share/awscredx/expirations.toml";

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

    pub fn get(profile: &str) -> Result<Option<DateTime<Utc>>, String> {
        let mut f = Self::read(util::path_to_absolute(EXPIRATIONS_FILE))?;
        Ok(f.0.remove(&ProfileName::new(profile)))
    }

    fn new() -> Self {
        Self(HashMap::new())
    }

    fn write(&self, path: &Path) -> Result<(), String> {
        let content = toml::to_string(&self.0).expect("Cannot encode expirations into TOML");
        util::create_storage_dir();
        fs::write(path, content).map_err(|e| format!("Cannot write file: {}", e))?;
        util::set_permissions(path, 0o600);
        Ok(())
    }
}
