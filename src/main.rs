use anyhow::{Context, Error, Ok, Result};
use aws_sdk_apigateway::types::{ApiKey, UsagePlanKey};
use console::{style, Style};
use dialoguer::{theme::ColorfulTheme, Confirm, FuzzySelect, Input, Password, Select};
use exodus::{
    delete_api_keys, get_api_keys, get_usage_plan_api_keys, get_usage_plans, init_client,
    put_api_keys, put_usage_plan_key, AwsCredentials, AWS_REGIONS,
};

fn get_aws_client_config(theme: &ColorfulTheme, regions: &'static [&'static str]) -> AwsCredentials {
    let aws_access_key_id: String = Input::with_theme(theme)
        .with_prompt("Access Key Id:")
        .interact()
        .unwrap();

    let aws_secret_access_key: String = Password::with_theme(theme)
        .with_prompt("Secret Access Key:")
        .interact()
        .unwrap();

    let aws_session_token: Option<String> = Some(
        Input::with_theme(theme)
            .with_prompt("Session Token:")
            .interact()
            .unwrap(),
    );

    let aws_region: String = FuzzySelect::with_theme(theme)
        .with_prompt("Pick your AWS Region")
        .items(regions)
        .default(0)
        .interact()
        .map(|index| regions[index].to_string())
        .unwrap();

    AwsCredentials::new(
        aws_access_key_id,
        aws_secret_access_key,
        aws_session_token,
        aws_region,
    )
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let theme = ColorfulTheme {
        values_style: Style::new().yellow(),
        ..ColorfulTheme::default()
    };

    let actions = vec!["Copy"];
    let selection = Select::with_theme(&theme)
        .with_prompt("What would you like to do?")
        .items(&actions)
        .interact()
        .context("Invalid action selected")
        .unwrap();
    let _selected_action = actions[selection];

    println!(
        "{}",
        style("Enter the AWS credentials for the account you want to copy from")
            .blue()
            .bright()
    );
    let aws_credentials = get_aws_client_config(&theme, &AWS_REGIONS);
    let source_client = init_client(aws_credentials).await;

    let from_usage_plan = Confirm::with_theme(&theme)
        .with_prompt("Would you like to only get API keys for a specific usage plan?")
        .interact()
        .unwrap();

    let usage_plans = if from_usage_plan {
        Some(get_usage_plans(&source_client).await?)
    } else {
        None
    };

    let source_plan = match &usage_plans {
        Some(plans) => {
            let plan_names = plans
                .iter()
                .map(|p| p.name().unwrap_or_default())
                .collect::<Vec<&str>>();

            let selection = Select::with_theme(&theme)
                .with_prompt("Select your usage plan")
                .items(&plan_names[..])
                .interact()
                .context("Invalid option selected")
                .unwrap();

            plans.get(selection)
        }
        None => None,
    };

    let use_query = Confirm::with_theme(&theme)
        .with_prompt("Do you want to filter api keys by names starting with specific text?")
        .interact()
        .unwrap();

    let name_query = if use_query {
        Some(
            Input::with_theme(&theme)
                .with_prompt("Enter the name query:")
                .interact()
                .unwrap(),
        )
    } else {
        None
    };

    let api_keys: Vec<ApiKey> = match source_plan {
        Some(usage_plan) => {
            let usage_plan_api_keys: Vec<UsagePlanKey> = get_usage_plan_api_keys(
                &source_client,
                usage_plan.id().unwrap_or_default(),
                name_query,
            )
            .await?;

            usage_plan_api_keys
                .into_iter()
                .map(|usage_plan_api_key| {
                    ApiKey::builder()
                        .id(usage_plan_api_key.id().unwrap())
                        .name(usage_plan_api_key.name().unwrap())
                        .value(usage_plan_api_key.value().unwrap())
                        .build()
                })
                .collect()
        }
        None => get_api_keys(&source_client, name_query).await?,
    };

    println!(
        "{}",
        style("Enter the AWS credentials for the account you want to paste to")
            .blue()
            .bright()
    );
    let aws_credentials = get_aws_client_config(&theme, &AWS_REGIONS);
    let destination_client = init_client(aws_credentials).await;

    let to_usage_plan = Confirm::with_theme(&theme)
        .with_prompt("Would you like to write the api keys to a specific usage plan?")
        .interact()
        .unwrap();

    let usage_plans = if to_usage_plan {
        Some(get_usage_plans(&destination_client).await?)
    } else {
        None
    };

    let destination_plan = match &usage_plans {
        Some(plans) => {
            let plan_names = plans
                .iter()
                .map(|p| p.name().unwrap_or_default())
                .collect::<Vec<&str>>();

            let selection = Select::with_theme(&theme)
                .with_prompt("Select your usage plan")
                .items(&plan_names[..])
                .interact()
                .context("Invalid option selected")
                .unwrap();

            plans.get(selection)
        }
        None => None,
    };

    let do_rename = Confirm::with_theme(&theme)
        .with_prompt("Would you like to rename the api keys?")
        .interact()
        .unwrap();

    let mut pattern: Option<String> = None;
    let mut replacement: Option<String> = None;
    if do_rename {
        pattern = Some(Input::with_theme(&theme)
            .with_prompt("Pattern to match in name (from start of string)")
            .interact()
            .unwrap());
        replacement = Some(Input::with_theme(&theme)
            .with_prompt("Replacement of matched pattern")
            .interact()
            .unwrap());
    }

    let created_keys = put_api_keys(&destination_client, &api_keys, pattern, replacement).await?;
    if destination_plan.is_some() {
        for key in created_keys {
            put_usage_plan_key(
                &destination_client,
                destination_plan
                    .unwrap()
                    .id()
                    .context("Invalid usage plan id")?,
                key.id().context("Invalid API key id")?,
            )
            .await?;
        }
    }

    let do_delete = Confirm::with_theme(&theme)
        .with_prompt("Would you like to delete the api keys in the source account?")
        .interact()
        .unwrap();

    if do_delete {
        delete_api_keys(&source_client, &api_keys).await;
    }

    Ok(())
}
