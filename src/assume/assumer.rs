use rusoto_sts::{StsClient, Sts, AssumeRoleRequest, GetSessionTokenRequest, NewAwsCredsForStsCreds};
use rusoto_core::{Region, HttpClient};
use rusoto_credential::{AwsCredentials, StaticProvider};
use crate::config::{Config, AssumeSubject};
use crate::credentials::{ProfileName, CredentialsFile};
use std::error::Error;
use chrono::{Utc, Duration};
use hyper_proxy::{Proxy, Intercept, ProxyConnector};
use hyper_tls::HttpsConnector;
use hyper::Uri;
use hyper::client::HttpConnector;
use crate::util;

pub struct RoleAssumer<'a> {
    region: Region,
    store: CredentialsFile,
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
    pub fn new(region: Region, store: CredentialsFile, config: &'a Config) -> Self {
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
                Some(exp) if *exp - Utc::now() < Duration::minutes(10) => // TODO: make the time configurable
                    self.get_new_credentials(profile),
                _ => Ok(cred.into()),
            }
            None => self.get_new_credentials(profile),
        }
    }

    fn get_new_credentials(&mut self, profile: &ProfileName) -> Result<Cred, String> {
        let parent = self.config.parent_profile(profile)
            .ok_or(format!("profile '{}' does not exist", &profile))?
            .clone();
        let parent_cred = self.profile_credentials(&parent)?;
        let sub = self.config.assume_subject(profile)?
            .ok_or(format!("cannot get assume subject for profile {}", profile))?;
        let parent_client = create_client(parent_cred, self.region.clone())?;
        let new_cred = assume_subject(&parent_client, sub)?;
        let out_cred = (&new_cred).into();
        self.store.put_credentials(profile.clone(), new_cred);
        Ok(out_cred)
    }
}

fn assume_subject(client: &StsClient, subject: AssumeSubject) -> Result<AwsCredentials, String> {
    let cred = match subject {
        AssumeSubject::Role { role_arn, session_name } => {
            let mut req = AssumeRoleRequest::default();
            req.role_arn = role_arn;
            req.role_session_name = session_name;
            let result = client.assume_role(req).sync()
                .map_err(|e| format!("unable to assume role: {}", e))?;
            result.credentials.expect("STS successful response contains None credentials")
        }
        AssumeSubject::MfaSession { serial_number, token_code } => {
            let mut req = GetSessionTokenRequest::default();
            req.serial_number = Some(serial_number);
            req.token_code = Some(token_code);
            let result = client.get_session_token(req).sync()
                .map_err(|e| format!("Unable to get MFA session token: {}", e.description()))?;

            result.credentials.expect("STS successful response contains None credentials")
        }
    };

    AwsCredentials::new_for_credentials(cred)
        .map_err(|e| format!("Cannot create AwsCredentials from STS credentials: {}", e))
}


fn create_client(credentials: Cred, region: Region) -> Result<StsClient, String> {
    Ok(StsClient::new_with(
        HttpClient::from_connector(get_https_connector()?),
        StaticProvider::new(
            credentials.key,
            credentials.secret,
            credentials.token,
            None,
        ),
        region,
    ))
}

fn get_https_connector() -> Result<ProxyConnector<HttpsConnector<HttpConnector>>, String> {
    let connector = HttpsConnector::new(2)
        .expect("connector with 2 threads");
    Ok(match util::get_https_proxy() {
        Some(proxy_url) => {
            let url = proxy_url.parse::<Uri>()
                .map_err(|e| format!("cannot parse proxy URL({}): {}", &proxy_url, e))?;
            let proxy = Proxy::new(Intercept::All, url);
            ProxyConnector::from_proxy(connector, proxy).expect("proxy created")
        }
        None => ProxyConnector::new(connector).expect("transparent proxy created")
    })
}