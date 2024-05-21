use anyhow::{Error, Ok, Result};
use clap::Parser;
use exodus::{delete_api_keys, get_api_keys, init_client, put_api_keys, Cli};
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Cli::parse();

    println!("Retrieving api keys from the source AWS account.");
    let src_account_client = init_client().await;
    let api_keys = get_api_keys(&src_account_client, args.name_query).await?;

    println!("\nCreating new api keys in the destination AWS account.");
    let dest_account_client = init_client().await;
    let _ = put_api_keys(
        &dest_account_client,
        &api_keys,
        args.pattern,
        args.replacement,
    )
    .await?;

    print!("\nDelete keys from the source account (y/n)? ");
    let _ = io::stdout().flush();
    let mut delete = String::new();
    io::stdin()
        .read_line(&mut delete)
        .expect("Failed to read input");

    if delete.trim() == "y" {
        println!("Deleting old api keys from the source account.");
        let _ = delete_api_keys(&src_account_client, &api_keys).await;
    }

    Ok(())
}
