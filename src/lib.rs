use anyhow::{Context, Ok, Result};
use aws_config::{BehaviorVersion, Region};
use aws_credential_types::Credentials;
use aws_sdk_apigateway::operation::create_usage_plan_key::CreateUsagePlanKeyOutput;
use aws_sdk_apigateway::types::{ApiKey, UsagePlanKey};
use aws_sdk_apigateway::Client;
use aws_sdk_apigateway::{operation::create_api_key::CreateApiKeyOutput, types::UsagePlan};
use console::style;
use regex::Regex;

pub const AWS_REGIONS: &'static [&'static str] = &[
    "us-east-1",
    "us-east-2",
    "us-west-1",
    "us-west-2",
    "af-south-1",
    "ap-east-1",
    "ap-south-2",
    "ap-southeast-3",
    "ap-southeast-4",
    "ap-south-1",
    "ap-northeast-3",
    "ap-northeast-2",
    "ap-southeast-1",
    "ap-southeast-2",
    "ap-northeast-1",
    "ca-central-1",
    "ca-west-1",
    "eu-central-1",
    "eu-west-1",
    "eu-west-2",
    "eu-south-1",
    "eu-west-3",
    "eu-south-2",
    "eu-north-1",
    "eu-central-2",
    "il-central-1",
    "me-south-1",
    "me-central-1",
    "sa-east-1",
    "us-gov-east-1",
    "us-gov-west-1",
];


pub struct AwsCredentials {
    access_key_id: String,
    secret_access_key: String,
    session_token: Option<String>,
    region: String,
}

impl AwsCredentials {
    pub fn new(
        access_key_id: String,
        secret_access_key: String,
        session_token: Option<String>,
        region: String,
    ) -> Self {
        Self {
            access_key_id,
            secret_access_key,
            session_token,
            region,
        }
    }
}

pub async fn get_usage_plans(client: &Client) -> Result<Vec<UsagePlan>> {
    let usage_plans = client
        .get_usage_plans()
        .into_paginator()
        .items()
        .send()
        .collect::<Result<Vec<UsagePlan>, _>>()
        .await?;

    println!(
        "{}",
        style(format!("Found {} usage plans", usage_plans.len()))
            .italic()
            .cyan()
    );
    Ok(usage_plans)
}

pub async fn get_api_keys(client: &Client, name_query: Option<String>) -> Result<Vec<ApiKey>> {
    let api_keys = client
        .get_api_keys()
        .include_values(true)
        .name_query(name_query.unwrap_or_default())
        .into_paginator()
        .items()
        .send()
        .collect::<Result<Vec<ApiKey>, _>>()
        .await?;

    println!(
        "{}",
        style(format!("Found {} api keys", api_keys.len()))
            .italic()
            .cyan()
    );
    Ok(api_keys)
}

pub async fn get_usage_plan_api_keys(
    client: &Client,
    usage_plan_id: &str,
    name_query: Option<String>,
) -> Result<Vec<UsagePlanKey>> {
    let api_keys = client
        .get_usage_plan_keys()
        .usage_plan_id(usage_plan_id)
        .name_query(name_query.unwrap_or_default())
        .into_paginator()
        .items()
        .send()
        .collect::<Result<Vec<UsagePlanKey>, _>>()
        .await?;

    Ok(api_keys)
}

pub async fn put_api_keys(
    client: &Client,
    api_keys: &Vec<ApiKey>,
    pattern: Option<String>,
    replacement: Option<String>,
) -> Result<Vec<CreateApiKeyOutput>> {
    let mut creation_results: Vec<CreateApiKeyOutput> = Vec::<CreateApiKeyOutput>::new();

    for api_key in api_keys {
        let res = put_api_key(client, api_key, &pattern, &replacement).await;
        creation_results.push(res.context(format!(
            "Failed to create api key \"{}\"",
            &api_key.name().unwrap_or_default()
        ))?);
    }

    println!(
        "{}",
        style(format!("Created {} api keys", creation_results.len()))
            .italic()
            .cyan()
    );
    Ok(creation_results)
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

pub async fn put_usage_plan_key(
    client: &Client,
    usage_plan_id: &str,
    api_key_id: &str,
) -> Result<CreateUsagePlanKeyOutput> {
    client
        .create_usage_plan_key()
        .usage_plan_id(usage_plan_id)
        .key_id(api_key_id)
        .key_type("API_KEY")
        .send()
        .await
        .context("Failed to create new usage plan key")
}

pub async fn delete_api_keys(client: &Client, api_keys: &Vec<ApiKey>) {
    for api_key in api_keys {
        let _ = client
            .delete_api_key()
            .api_key(api_key.id().unwrap())
            .send()
            .await;
    }

    println!(
        "{}",
        style(format!("Deleted {} api keys", api_keys.len()))
            .italic()
            .cyan()
    );
}

pub async fn init_client(credentials: AwsCredentials) -> Client {
    let credentials_provider = Credentials::from_keys(
        credentials.access_key_id,
        credentials.secret_access_key,
        credentials.session_token,
    );

    let config = aws_config::defaults(BehaviorVersion::latest())
        .region(Region::new(credentials.region))
        .credentials_provider(credentials_provider)
        .load()
        .await;

    apigateway_client(&config)
}

fn apigateway_client(config: &aws_config::SdkConfig) -> Client {
    let apigateway_config_builder = aws_sdk_apigateway::config::Builder::from(config);
    Client::from_conf(apigateway_config_builder.build())
}
