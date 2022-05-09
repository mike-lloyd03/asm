use anyhow::Result;
use clap::{Parser, Subcommand};
use colored_json::to_colored_json_auto;
use serde::Deserialize;
use serde::Serialize;
use std::process::exit;
use tabled::{Alignment, Full, Modify, Style, Table, Tabled};

use asm::AwsSM;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Get the ARN of a secret
    GetArn {
        /// The string to search on
        search_string: String,
    },
    /// Get the value of a secret
    #[clap(alias("g"))]
    #[clap(alias("get"))]
    GetValue {
        /// The string to search on
        search_string: String,
    },
    /// Search for secrets by name
    #[clap(alias("s"))]
    Search {
        /// The string to search on
        search_string: String,
    },

    /// List all available secrets
    #[clap(alias("l"))]
    List,
}

#[derive(Debug, Deserialize)]
struct SecretList {
    #[serde(rename = "SecretList")]
    list: Vec<Secret>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Secret {
    #[serde(rename = "ARN")]
    arn: String,
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Description")]
    description: Option<String>,
    #[serde(rename = "SecretString")]
    value: Option<String>,
}

impl Tabled for Secret {
    const LENGTH: usize = 2;

    fn fields(&self) -> Vec<String> {
        let desc = match &self.description {
            Some(d) => d,
            None => "",
        };
        vec![self.name.clone(), desc.to_string()]
    }

    fn headers() -> Vec<String> {
        vec!["Name".to_string(), "Description".to_string()]
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::GetArn { search_string } => {
            println!("{}", select_secret(search_string.to_string()).unwrap().arn);
        }
        Commands::GetValue { search_string } => {
            search_and_get_value(search_string.to_string()).unwrap()
        }
        Commands::Search { search_string } => search_cmd(search_string.to_string()).unwrap(),
        Commands::List => list_cmd(),
    };
}

/// Prints a list of secrets containing `search_string`
fn search_cmd(search_string: String) -> Result<()> {
    let secrets = search_secrets(search_string)?;

    check_results(&secrets);
    print_secret_table(&secrets);
    Ok(())
}

fn print_secret_table(secrets: &SecretList) {
    let table = Table::new(&secrets.list)
        .with(Style::psql())
        .with(Modify::new(Full).with(Alignment::left()));
    println!("\n{}\n", table);
}

/// Gets a secret value by searching for secrets containing `search_string` and allowing the
/// user to choose which secret to retrieve if more than one secret is found.
fn search_and_get_value(search_string: String) -> Result<()> {
    let selected_secret = select_secret(search_string)?;
    println!("{}", get_secret_value(&selected_secret.arn)?);
    Ok(())
}

/// Returns a list of secrets containing `search_string` in their name.
fn search_secrets(search_string: String) -> Result<SecretList> {
    let output = AwsSM::new("list-secrets").run();
    let mut secrets: SecretList = serde_json::from_str(&output)?;
    secrets.list.retain(|s| {
        s.name
            .to_lowercase()
            .contains(&search_string.to_lowercase())
    });
    Ok(secrets)
}

/// Returns a secret from a list of secrets based on `search_string`. If more than one secret is
/// returned from the search, the user is prompted to choose one.
fn select_secret(search_string: String) -> Result<Secret> {
    let mut secrets = search_secrets(search_string)?;
    let i: usize;

    check_results(&secrets);

    if secrets.list.len() > 1 {
        eprintln!("Multiple secrets were found");
        secrets.list.iter().enumerate().for_each(|(i, s)| {
            eprintln!("{}: {}", i, s.name);
        });
        eprint!("\nSelect secret: ");
        match read_int() {
            Ok(input) => {
                let max_idx = secrets.list.len() - 1;
                if input <= max_idx {
                    i = input;
                } else {
                    eprintln!("Please enter a value between 0 and {}", max_idx);
                    exit(1);
                }
            }
            Err(_) => {
                eprintln!("Please enter an integer value");
                exit(1);
            }
        }
    } else {
        i = 0;
    }
    Ok(secrets.list.remove(i))
}

/// Returns the secret value for a given ARN.
fn get_secret_value(arn: &str) -> Result<String> {
    let output = AwsSM::new("get-secret-value")
        .args(["--secret-id", arn])
        .run();
    let secret: Secret = serde_json::from_str(&output)?;

    // If the secret value is json serializable, parse it. Otherwise, return the string
    Ok(
        match serde_json::from_str::<serde_json::Value>(
            secret.value.as_ref().unwrap_or(&"".to_string()),
        ) {
            Ok(s) => to_colored_json_auto(&s)?,
            Err(_) => secret.value.unwrap_or_else(|| "".to_string()),
        },
    )
}

/// Reads a usize integer from stdin.
fn read_int() -> Result<usize> {
    use std::io::{stdin, stdout, Write};
    let mut input = String::new();

    stdout().flush()?;
    stdin().read_line(&mut input)?;
    Ok(input.trim().parse()?)
}

fn check_results(secrets: &SecretList) {
    if secrets.list.is_empty() {
        eprintln!("Your search did not return any results");
        exit(1);
    }
}

fn list_cmd() {
    let output = AwsSM::new("list-secrets").run();
    let secrets: SecretList = serde_json::from_str(&output).unwrap();
    print_secret_table(&secrets);
}
