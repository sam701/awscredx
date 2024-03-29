use ansi_term::{Color, Style};
use chrono::{Duration, TimeZone, Utc};
use rusoto_core::{HttpClient, Region};
use rusoto_credential::{AwsCredentials, StaticProvider};
use rusoto_iam::{CreateAccessKeyRequest, DeleteAccessKeyRequest, Iam, IamClient};

use crate::config::Config;
use crate::credentials::CredentialsFile;
use crate::state::State;

pub fn rotate_if_needed(
    config: &Config,
    cred_file: &mut CredentialsFile,
    state: &mut State,
) -> Result<(), String> {
    if let Some(days) = config.rotate_credentials_days {
        let now = Utc::now();
        let last_rotation = state
            .last_credentials_rotation_time
            .unwrap_or_else(|| Utc.timestamp(0, 0));
        if now - last_rotation >= Duration::days(days) {
            rotate_credentials(cred_file, config)?;
            state.last_credentials_rotation_time = Some(now);
            state.save()?;
        }
    }
    Ok(())
}

fn rotate_credentials(cred_file: &mut CredentialsFile, config: &Config) -> Result<(), String> {
    let (client, access_key) = {
        let cred = cred_file
            .get_credentials(&config.main_profile)
            .ok_or(format!(
                "cannot get credentials for main profile '{}'",
                config.main_profile.as_ref()
            ))?;
        (
            create_iam_client(cred)?,
            cred.aws_access_key_id().to_owned(),
        )
    };

    eprintln!(
        "{}: access key is more than {} days old.",
        Style::new()
            .fg(Color::Yellow)
            .bold()
            .paint("Rotating Access Key"),
        config.rotate_credentials_days.unwrap()
    );
    eprint!("  Creating new access key... ");
    let runtime = super::create_runtime();
    let client2 = client.clone();
    let ak_resp = runtime.block_on(async move {
        client2
            .create_access_key(CreateAccessKeyRequest { user_name: None })
            .await
            .map_err(|e| format!("cannot create new IAM access key: {}", e))
    })?;
    let ok_style = Style::new().fg(Color::Green).bold();
    eprintln!("{}", ok_style.paint("ok"));

    cred_file.put_credentials(
        config.main_profile.clone(),
        AwsCredentials::new(
            ak_resp.access_key.access_key_id,
            ak_resp.access_key.secret_access_key,
            None,
            None,
        ),
    );
    cred_file.write()?;

    eprint!("  Deleting old access key... ");
    runtime.block_on(async move {
        client
            .delete_access_key(DeleteAccessKeyRequest {
                access_key_id: access_key.clone(),
                user_name: None,
            })
            .await
            .map_err(|e| format!("cannot delete old access key({}): {}", &access_key, e))
    })?;
    eprintln!("{}", ok_style.paint("ok"));

    Ok(())
}

fn create_iam_client(credentials: &AwsCredentials) -> Result<IamClient, String> {
    Ok(IamClient::new_with(
        HttpClient::from_connector(super::get_https_connector()?),
        StaticProvider::new(
            credentials.aws_access_key_id().to_owned(),
            credentials.aws_secret_access_key().to_owned(),
            credentials.token().clone(),
            None,
        ),
        Region::UsEast1,
    ))
}
