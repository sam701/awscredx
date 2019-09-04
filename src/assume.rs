use rusoto_sts::{StsClient, Sts, AssumeRoleRequest, GetSessionTokenRequest, NewAwsCredsForStsCreds};
use rusoto_core::{Region, HttpClient};
use rusoto_credential::{AwsCredentials, StaticProvider};

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

pub struct RoleAssumer {
    region: Region,
    store: Box<dyn CredentialsStore>,
    meta_data_provider: Box<dyn ProfileMetaDataProvider>,
}

#[derive(Clone)]
pub struct Profile(String);

pub trait CredentialsStore {
    fn get_credentials(&self, profile: &Profile) -> Option<&AwsCredentials>;
    fn put_credentials(&mut self, profile: Profile, credentials: AwsCredentials) -> &AwsCredentials;
    fn persist(&self) -> Result<(), String>;
}

pub trait ProfileMetaDataProvider {
    fn get_source_profile(&self, profile: &Profile) -> Option<&Profile>;
    fn get_assume_subject(&self, profile: &Profile) -> AssumeSubject;
}

struct CredentialsChain {
    initial: AwsCredentials,
    profiles: Vec<Profile>,
}

impl RoleAssumer {
    pub fn new(region: Region, store: Box<dyn CredentialsStore>, meta_data_provider: Box<dyn ProfileMetaDataProvider>) -> Self {
        Self {
            region,
            store,
            meta_data_provider,
        }
    }

    fn assume_chain(&mut self, chain: CredentialsChain) -> Result<(), String> {
        let mut cred = &chain.initial;
        for p in chain.profiles {
            let client = create_client(cred, self.region.clone());
            let new_cred = assume_subject(&client, self.meta_data_provider.get_assume_subject(&p))?;
            cred = self.store.put_credentials(p, new_cred);
        }
        self.store.persist()
    }

    fn build_cred_chain(&self, profile: Profile) -> Result<CredentialsChain, String> {
        let mut p = profile;
        let mut list = Vec::new();
        let mut result = None;
        loop {
            match self.store.get_credentials(&p) {
                None => {
                    list.push(p.clone());
                    p = self.meta_data_provider.get_source_profile(&p)
                        .map(|x| x.clone())
                        .ok_or(format!("Cannot get source profile for {}", &p.0))?;
                }
                Some(cred) => {
                    result = Some(CredentialsChain {
                        initial: cred.clone(),
                        profiles: list,
                    });
                    break;
                }
            }
        }
        Ok(result.unwrap())
    }

    pub fn assume(&mut self, profile: Profile) -> Result<(), String> {
        let chain = self.build_cred_chain(profile)?;
        self.assume_chain(chain)
    }
}

fn assume_subject(client: &StsClient, subject: AssumeSubject) -> Result<AwsCredentials, String> {
    let cred = match subject {
        AssumeSubject::Role { role_arn, session_name } => {
            let mut req = AssumeRoleRequest::default();
            req.role_arn = role_arn;
            req.role_session_name = session_name;
            let result = client.assume_role(req).sync()
                .map_err(|e| format!("Unable to assume role: {}", e))?;
            result.credentials.expect("STS successful response contains None credentials")
        }
        AssumeSubject::MfaSession { serial_number, token_code } => {
            let mut req = GetSessionTokenRequest::default();
            req.serial_number = Some(serial_number.to_string());
            req.token_code = Some(token_code.to_string());
            let result = client.get_session_token(req).sync()
                .map_err(|e| format!("Unable to get MFA session token: {}", e))?;

            result.credentials.expect("STS successful response contains None credentials")
        }
    };

    AwsCredentials::new_for_credentials(cred)
        .map_err(|e| format!("Cannot create AwsCredentials from STS credentials: {}", e))
}


fn create_client(credentials: &AwsCredentials, region: Region) -> StsClient {
    StsClient::new_with(
        HttpClient::new().unwrap(),
        StaticProvider::new(
            credentials.aws_access_key_id().to_owned(),
            credentials.aws_secret_access_key().to_owned(),
            credentials.token().clone(),
            None,
        ),
        region,
    )
}
