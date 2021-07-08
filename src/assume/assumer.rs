use chrono::{Duration, Utc};
use rusoto_core::{HttpClient, Region};
use rusoto_credential::{AwsCredentials, StaticProvider};
use rusoto_sts::{
    AssumeRoleRequest, GetSessionTokenRequest, NewAwsCredsForStsCreds, Sts, StsClient,
};

use crate::config::{AssumeSubject, Config};
use crate::credentials::{CredentialsFile, ProfileName};

pub struct RoleAssumer<'a> {
    region: Region,
    store: &'a mut CredentialsFile,
    config: &'a Config,
}

struct Cred {
    key: String,
    secret: String,
    token: Option<String>,
}

impl From<&AwsCredentials> for Cred {
    fn from(c: &AwsCredentials) -> Self {
        Cred {
            key: c.aws_access_key_id().to_owned(),
            secret: c.aws_secret_access_key().to_owned(),
            token: c.token().clone(),
        }
    }
}

impl<'a> RoleAssumer<'a> {
    pub fn new(region: Region, store: &'a mut CredentialsFile, config: &'a Config) -> Self {
        Self {
            region,
            store,
            config,
        }
    }

    pub fn assume(&mut self, profile: &str) -> Result<(), String> {
        let pn = ProfileName::new(profile.to_owned());
        self.profile_credentials(&pn).map(|_| ())?;
        self.store.write()
    }

    fn profile_credentials(&mut self, profile: &ProfileName) -> Result<Cred, String> {
        match self.store.get_credentials(&profile) {
            Some(cred) => match cred.expires_at() {
                Some(exp) if *exp - Utc::now() < Duration::minutes(10) =>
                // TODO: make the time configurable
                {
                    self.get_new_credentials(profile)
                }
                _ => Ok(cred.into()),
            },
            None => self.get_new_credentials(profile),
        }
    }

    fn get_new_credentials(&mut self, profile: &ProfileName) -> Result<Cred, String> {
        let parent = self
            .config
            .parent_profile(profile)
            .ok_or(format!("profile '{}' does not exist", &profile))?
            .clone();
        let parent_cred = self.profile_credentials(&parent)?;
        let sub = self
            .config
            .assume_subject(profile)?
            .ok_or(format!("cannot get assume subject for profile {}", profile))?;
        let parent_client = create_sts_client(parent_cred, self.region.clone())?;
        let new_cred = assume_subject(&parent_client, sub)?;
        let out_cred = (&new_cred).into();
        self.store.put_credentials(profile.clone(), new_cred);
        Ok(out_cred)
    }
}

fn assume_subject(client: &StsClient, subject: AssumeSubject) -> Result<AwsCredentials, String> {
    let cred = match subject {
        AssumeSubject::Role {
            role_arn,
            session_name,
        } => {
            let req = AssumeRoleRequest {
                role_arn,
                role_session_name: session_name,
                ..Default::default()
            };
            let result = client
                .assume_role(req)
                .sync()
                .map_err(|e| format!("unable to assume role: {}", e))?;
            result
                .credentials
                .expect("STS successful response contains None credentials")
        }
        AssumeSubject::MfaSession {
            serial_number,
            token_code,
        } => {
            let req = GetSessionTokenRequest {
                serial_number: Some(serial_number),
                token_code: Some(token_code),
                duration_seconds: None,
            };
            let result = client
                .get_session_token(req)
                .sync()
                .map_err(|e| format!("Unable to get MFA session token: {}", e))?;

            result
                .credentials
                .expect("STS successful response contains None credentials")
        }
    };

    AwsCredentials::new_for_credentials(cred)
        .map_err(|e| format!("Cannot create AwsCredentials from STS credentials: {}", e))
}

fn create_sts_client(credentials: Cred, region: Region) -> Result<StsClient, String> {
    Ok(StsClient::new_with(
        HttpClient::from_connector(super::get_https_connector()?),
        StaticProvider::new(credentials.key, credentials.secret, credentials.token, None),
        region,
    ))
}
