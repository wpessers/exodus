use anyhow::{Context, Error, Ok, Result};
use aws_config::BehaviorVersion;
use aws_sdk_apigateway::operation::create_api_key::CreateApiKeyOutput;
use aws_sdk_apigateway::types::ApiKey;
use aws_sdk_apigateway::Client;
use clap::Parser;
use regex::Regex;
use std::env;
use std::io::{self, Write};

const AWS_CREDENTIALS: &[(&str, &str)] = &[
    ("AWS_ACCESS_KEY_ID", "AWS Access Key Id"),
    ("AWS_SECRET_ACCESS_KEY", "AWS Secret Access Key"),
    ("AWS_SESSION_TOKEN", "AWS Session token"),
    ("AWS_REGION", "AWS Region"),
];

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

fn set_aws_credentials() {
    println!("Enter your AWS credentials");

    for credential in AWS_CREDENTIALS {
        set_env_var(credential.0, credential.1);
    }
}

fn set_env_var(env_var: &str, prompt: &str) {
    print!("{prompt}: ");
    let _ = io::stdout().flush();

    let mut env_var_value = String::new();
    io::stdin()
        .read_line(&mut env_var_value)
        .expect("Failed to read input");

    env::set_var(env_var, env_var_value.trim());
}

pub async fn get_api_keys(
    client: &Client,
    name_query: Option<String>,
) -> Result<Vec<ApiKey>, Error> {
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
) {
    println!("Creating {} api keys", api_keys.len());

    for api_key in api_keys {
        let res = put_api_key(client, api_key, pattern.clone(), replacement.clone()).await;
        println!(
            "Created: {}",
            res.context(format!(
                "Failed to create api key \"{}\"",
                &api_key.name().unwrap()
            ))
            .unwrap()
            .name()
            .unwrap()
        );
    }
}

async fn put_api_key(
    client: &Client,
    api_key: &ApiKey,
    pattern: Option<String>,
    replacement: Option<String>,
) -> Result<CreateApiKeyOutput, Error> {
    let mut api_key_name = api_key.name().context("Invalid API key name")?.to_owned();
    if let (Some(pattern), Some(replacement)) = (pattern, replacement) {
        let re = Regex::new(&pattern).unwrap();
        api_key_name = re.replace(&api_key_name, &replacement).to_string();
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
    set_aws_credentials();
    let config = aws_config::defaults(BehaviorVersion::latest())
        .endpoint_url("http://localhost:4566/")
        .load()
        .await;
    apigateway_client(&config)
}

fn apigateway_client(config: &aws_config::SdkConfig) -> Client {
    let apigateway_config_builder = aws_sdk_apigateway::config::Builder::from(config);
    Client::from_conf(apigateway_config_builder.build())
}
