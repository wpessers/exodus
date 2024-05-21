use anyhow::{Context, Ok, Result};
use aws_config::{BehaviorVersion, Region};
use aws_credential_types::Credentials;
use aws_sdk_apigateway::operation::create_api_key::CreateApiKeyOutput;
use aws_sdk_apigateway::types::ApiKey;
use aws_sdk_apigateway::Client;
use clap::Parser;
use regex::Regex;
use rpassword::read_password;
use std::io::{self, Write};

#[derive(Parser)]
pub struct Cli {
    /// Name query to match API key names starting with this exact value
    #[arg(long, short)]
    pub name_query: Option<String>,

    /// Pattern of part of the API key names to match to be replaced with a new name
    #[arg(long, short)]
    pub pattern: Option<String>,

    /// New name of the API keys matching the pattern specified before
    #[arg(long, short)]
    pub replacement: Option<String>,
}

struct AwsCredentials {
    access_key_id: String,
    secret_access_key: String,
    session_token: Option<String>,
    region: String,
}

pub async fn get_api_keys(client: &Client, name_query: Option<String>) -> Result<Vec<ApiKey>> {
    let api_keys = client
        .get_api_keys()
        .include_values(true)
        .name_query(name_query.unwrap_or_default())
        .into_paginator()
        .items()
        .send()
        .collect::<Result<Vec<_>, _>>()
        .await?;

    println!("Found {} api keys", api_keys.len());
    Ok(api_keys)
}

pub async fn put_api_keys(
    client: &Client,
    api_keys: &Vec<ApiKey>,
    pattern: Option<String>,
    replacement: Option<String>,
) -> Result<Vec<CreateApiKeyOutput>> {
    let mut results: Vec<CreateApiKeyOutput> = Vec::<CreateApiKeyOutput>::new();
    println!("Creating {} api keys", api_keys.len());

    for api_key in api_keys {
        let res = put_api_key(client, api_key, &pattern, &replacement).await;
        results.push(res.context(format!(
            "Failed to create api key \"{}\"",
            &api_key.name().unwrap_or_default()
        ))?);
    }

    println!("Created {} api keys", results.len());
    Ok(results)
}

async fn put_api_key(
    client: &Client,
    api_key: &ApiKey,
    pattern: &Option<String>,
    replacement: &Option<String>,
) -> Result<CreateApiKeyOutput> {
    let mut api_key_name = api_key.name().context("Invalid API key name")?.to_owned();
    if let (Some(pattern), Some(replacement)) = (pattern, replacement) {
        let re = Regex::new(pattern).context("Invalid regex pattern")?;
        api_key_name = re.replace(&api_key_name, replacement).to_string();
    }

    client
        .create_api_key()
        .name(api_key_name)
        .value(api_key.value().context("Invalid api key value")?)
        .description(api_key.description().unwrap_or_default())
        .enabled(api_key.enabled())
        .send()
        .await
        .context("Failed to create new api key")
}

pub async fn delete_api_keys(client: &Client, api_keys: &Vec<ApiKey>) {
    for api_key in api_keys {
        let _ = client
            .delete_api_key()
            .api_key(api_key.id().unwrap())
            .send()
            .await;
    }
    println!("Deleted {} keys", api_keys.len());
}

pub async fn init_client() -> Client {
    let credentials = set_aws_credentials();
    let credentials_provider = Credentials::from_keys(
        credentials.access_key_id,
        credentials.secret_access_key,
        credentials.session_token,
    );

    let config = aws_config::defaults(BehaviorVersion::latest())
        .endpoint_url("http://localhost:4566/")
        .region(Region::new(credentials.region))
        .credentials_provider(credentials_provider)
        .load()
        .await;

    apigateway_client(&config)
}

fn set_aws_credentials() -> AwsCredentials {
    println!("Enter your AWS credentials");
    let access_key_id = prompt("Access Key Id", false);
    let secret_access_key = prompt("Secret Access Key", true);
    let session_token = Some(prompt("Session Token", true));
    let region = prompt("Region", false);

    AwsCredentials {
        access_key_id,
        secret_access_key,
        session_token,
        region,
    }
}

fn prompt(prompt: &str, sensitive: bool) -> String {
    let hidden = match sensitive {
        true => " (hidden)",
        false => "",
    };

    print!("{prompt}{hidden}: ");
    let _ = io::stdout().flush();
    let mut input_value = String::new();

    if sensitive {
        input_value = read_password().expect("Failed to read input");
    } else {
        io::stdin()
            .read_line(&mut input_value)
            .expect("Failed to read input");
    }

    input_value.trim().to_string()
}

fn apigateway_client(config: &aws_config::SdkConfig) -> Client {
    let apigateway_config_builder = aws_sdk_apigateway::config::Builder::from(config);
    Client::from_conf(apigateway_config_builder.build())
}
