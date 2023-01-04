use std::process::exit;

use anyhow::Result;
use colored_json::to_colored_json_auto;
use serde::Deserialize;
use serde_json::Value;
use tabled::{object::Segment, Alignment, Modify, Style, Table};

mod aws_sm;
mod secret;

pub use aws_sm::AwsSM;
pub use secret::*;

/// Prints a list of secrets containing `search_string`
pub fn search_cmd(search_string: String) -> Result<()> {
    let secrets = search_secrets(&search_string)?;

    check_results(&secrets);
    print_secret_table(&secrets);
    Ok(())
}

/// Prints a table with a list of secrets
pub fn print_secret_table(secrets: &Vec<Secret>) {
    let table = Table::new(secrets)
        .with(Style::rounded())
        .with(Modify::new(Segment::all()).with(Alignment::left()));
    println!("\n{}\n", table);
}

/// Gets a secret value by searching for secrets containing `search_string` and allowing the
/// user to choose which secret to retrieve if more than one secret is found.
pub fn search_and_get_value(search_string: String) -> Result<()> {
    let selected_secret = select_secret(&search_string)?;
    println!("{}", get_secret_value(&selected_secret.arn)?);
    Ok(())
}

/// Returns a list of secrets containing `search_string` in their name.
fn search_secrets(search_string: &str) -> Result<Vec<Secret>> {
    #[derive(Debug, Deserialize)]
    pub struct SecretList {
        #[serde(rename = "SecretList")]
        pub list: Vec<Secret>,
    }

    let output = AwsSM::new("list-secrets").run();
    let secret_list: SecretList = serde_json::from_str(&output)?;
    let mut secrets: Vec<Secret> = secret_list.list;

    secrets.retain(|s| {
        s.name
            .to_lowercase()
            .contains(&search_string.to_lowercase())
    });
    Ok(secrets)
}

/// Returns a secret from a list of secrets based on `search_string`. If more than one secret is
/// returned from the search, the user is prompted to choose one.
pub fn select_secret(search_string: &str) -> Result<Secret> {
    let mut secrets = search_secrets(search_string)?;
    let i: usize;

    check_results(&secrets);

    if secrets.len() > 1 {
        eprintln!("Multiple secrets were found");
        secrets.iter().enumerate().for_each(|(i, s)| {
            eprintln!("{}: {}", i, s.name);
        });
        eprint!("\nSelect secret: ");
        match read_int() {
            Ok(input) => {
                let max_idx = secrets.len() - 1;
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
    Ok(secrets.remove(i))
}

/// Returns the secret value for a given ARN.
pub fn get_secret_value(arn: &str) -> Result<String> {
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

fn check_results(secrets: &Vec<Secret>) {
    if secrets.is_empty() {
        eprintln!("Your search did not return any results");
        exit(1);
    }
}

pub fn list_cmd() {
    let output = AwsSM::new("list-secrets").run();
    let secrets: Vec<Secret> = serde_json::from_str(&output).unwrap();
    print_secret_table(&secrets);
}
