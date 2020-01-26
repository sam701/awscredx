use std::{env, process};
use std::collections::HashMap;

use rusoto_credential::AwsCredentials;
use serde::Serialize;

use crate::config::Config;
use crate::credentials::{CredentialsFile, ProfileName};
use crate::util;

const SIGN_IN_URL: &str = "https://signin.aws.amazon.com/federation";

#[derive(Serialize)]
struct SessionData<'a> {
    #[serde(rename = "sessionId")]
    id: &'a str,

    #[serde(rename = "sessionKey")]
    key: &'a str,

    #[serde(rename = "sessionToken")]
    token: Option<&'a String>,
}

fn build_sign_in_token_url(cred: &AwsCredentials) -> String {
    let session_json = serde_json::to_string(&SessionData {
        id: cred.aws_access_key_id(),
        key: cred.aws_secret_access_key(),
        token: cred.token().as_ref(),
    }).unwrap();

    format!("{}?Action=getSigninToken&DurationSeconds={}&SessionType=json&{}",
            SIGN_IN_URL,
            3600,
            serde_urlencoded::to_string(&[("Session", session_json)]).unwrap())
}

pub fn create_signin_url(aws_service_name: &str, open_in_browser: bool) {
    if let Err(e) = create(aws_service_name, open_in_browser) {
        eprintln!("{}: {}", &util::styled_error_word(), e);
        process::exit(1);
    }
}

fn create(aws_service_name: &str, open_in_browser: bool) -> Result<(), String> {
    let cred_file = CredentialsFile::read_default()?;
    let profile = env::var("AWS_PROFILE")
        .map_err(|_e| "AWS_PROFILE is not set")?;
    let cred = cred_file.get_credentials(&ProfileName::new(&profile))
        .ok_or(format!("Profile {} is expired", profile))?;
    let sign_in_url = build_sign_in_token_url(cred);

    let client = util::get_https_client()?;
    let response: HashMap<String, String> = client.get(&sign_in_url)
        .send()
        .map_err(|e| format!("Error during calling sign in URL: {}", e))?
        .json()
        .map_err(|e| format!("Cannot decode AWS sign in response: {}", e))?;

    let sign_in_token = response.get("SigninToken")
        .ok_or("AWS sign in response does not contain sign in token")?;

    let config = Config::read()?.ok_or("Config file does not exist")?;
    let destination_url = format!("https://{region}.console.aws.amazon.com/{}/home?region={region}",
                                  aws_service_name, region = config.region.name());
    let sign_in_url = format!("{}?{}",
                              SIGN_IN_URL,
                              serde_urlencoded::to_string(&[
                                  ("Action", "login"),
                                  ("Issuer", ""),
                                  ("Destination", &destination_url),
                                  ("SigninToken", &sign_in_token),
                              ]).unwrap());
    if open_in_browser {
        webbrowser::open(&sign_in_url)
            .map_err(|e| format!("Cannot open URL in browser: {}", e))?;
    } else {
        println!("{}", &sign_in_url);
    }
    Ok(())
}