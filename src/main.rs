use anyhow::Result;
use clap::{Parser, Subcommand};
use colored_json::to_colored_json_auto;
use serde::Deserialize;
use serde::Serialize;
use std::process::exit;
use std::process::Command;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Cli {
    // #[clap(short, long)]
    // quiet_flag: bool,
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    GetArn,
    GetValue { search_string: String },
    // The string to search on
    Search { search_string: String },
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

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::GetArn => todo!(),
        Commands::GetValue { search_string } => search_and_get_value(search_string.to_string()),
        Commands::Search { search_string } => search_cmd(search_string.to_string()),
    }
}

/// Prints a list of secrets containing `search_string`
fn search_cmd(search_string: String) {
    let secrets = search_secrets(search_string).unwrap();

    for s in secrets.list {
        println!(
            "{}: {}",
            s.name,
            s.description.unwrap_or_else(|| "".to_string())
        );
    }
}

/// Gets a secret value by searching for secrets containing `search_string` and allowing the
/// user to choose which secret to retrieve if more than one secret is found.
fn search_and_get_value(search_string: String) {
    let secrets = search_secrets(search_string).unwrap();
    let i: usize;

    if secrets.list.len() > 1 {
        println!("Multiple secrets were found");
        secrets.list.iter().enumerate().for_each(|(i, s)| {
            println!("{}: {}", i, s.name);
        });
        print!("\nSelect secret: ");
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
                println!("Please enter an integer value");
                exit(1);
            }
        }
    } else {
        i = 0;
    }
    println!("{}", get_secret_value(&secrets.list[i].arn).unwrap())
}

/// Returns a list of secrets containing `search_string` in their name.
fn search_secrets(search_string: String) -> Result<SecretList> {
    let mut aws_sm = Command::new("aws");
    aws_sm.arg("secretsmanager").arg("list-secrets");

    let output = aws_sm.output()?;
    let output_str = std::str::from_utf8(&output.stdout)?;
    let mut secrets: SecretList = serde_json::from_str(output_str)?;
    secrets.list.retain(|s| {
        s.name
            .to_lowercase()
            .contains(&search_string.to_lowercase())
    });
    Ok(secrets)
}

/// Reads a usize integer from stdin.
fn read_int() -> Result<usize> {
    use std::io::{stdin, stdout, Write};
    let mut input = String::new();

    stdout().flush()?;
    stdin().read_line(&mut input)?;
    Ok(input.trim().parse()?)
}

/// Returns the secret value for a given ARN.
fn get_secret_value(arn: &str) -> Result<String> {
    let mut aws_sm = Command::new("aws");
    aws_sm
        .arg("secretsmanager")
        .arg("get-secret-value")
        .args(["--secret-id", arn]);
    let output = aws_sm.output()?;
    let output_str = std::str::from_utf8(&output.stdout)?;
    let secret: Secret = serde_json::from_str(output_str)?;
    Ok(
        match serde_json::from_str::<serde_json::Value>(secret.value.as_ref().unwrap()) {
            Ok(s) => to_colored_json_auto(&s)?,
            Err(_) => secret.value.unwrap_or_else(|| "".to_string()),
        },
    )
}
