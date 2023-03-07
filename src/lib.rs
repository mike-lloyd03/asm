use std::fs;
use std::io::{stdin, stdout, Write};
use std::{env, path::Path, process::exit};

use anyhow::Result;
use colored_json::to_colored_json_auto;
use mktemp::Temp;
use serde::Deserialize;
use serde_json::to_string_pretty;
use tabled::{object::Segment, Alignment, Modify, Style, Table};

mod aws_sm;
mod secret;

pub use aws_sm::AwsSM;
pub use secret::*;

#[derive(Debug, Deserialize)]
pub struct SecretList {
    #[serde(rename = "SecretList")]
    pub list: Vec<Secret>,
}

/// Returns a secret from a list of secrets based on `search_string`. If more than one secret is
/// returned from the search, the user is prompted to choose one.
pub fn select_secret(search_string: &str) -> Result<Secret> {
    let mut secrets = search_all_secrets(search_string)?;
    let i: usize;

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

/// Prints a list of secrets containing `search_string`
pub fn search_secret(search_string: &str) -> Result<()> {
    let secrets = search_all_secrets(search_string)?;

    print_secret_table(&secrets);
    Ok(())
}

/// Gets a secret value by searching for secrets containing `search_string` and allowing the
/// user to choose which secret to retrieve if more than one secret is found.
pub fn search_and_get_value(search_string: &str) -> Result<()> {
    let selected_secret = select_secret(search_string)?;
    println!("{}", get_secret_value(&selected_secret.arn, true)?);
    Ok(())
}

/// Lists all secrets
pub fn list_secrets() -> Result<()> {
    let output = AwsSM::new("list-secrets").run();
    let secrets: SecretList = serde_json::from_str(&output)?;
    print_secret_table(&secrets.list);
    Ok(())
}

/// Creates a new secret
pub fn create_secret(secret_name: &str, description: &Option<String>) -> Result<()> {
    let secret_file = Temp::new_path();
    let secret_file_path = format!(
        "file://{}",
        &secret_file.to_str().expect("temp file should have path")
    );

    edit_file(secret_file.as_path());

    if Path::exists(secret_file.as_path()) {
        let mut args = vec![
            "--name",
            secret_name,
            "--secret-string",
            secret_file_path.as_str(),
        ];
        if let Some(desc) = description {
            args.extend(vec!["--description", desc])
        }
        AwsSM::new("create-secret").args(args).run();

        println!("Created secret {}", secret_name);
    } else {
        println!("Aborting...")
    }

    Ok(())
}

/// Creates a new secret
pub fn delete_secret(search_string: &str) -> Result<()> {
    let secret = select_secret(search_string)?;
    print!(
        "Are you sure you want to delete secret '{}' [y/N]? ",
        secret.name
    );
    let resp = read_string()?;
    if resp == "y" || resp.to_lowercase().starts_with("yes") {
        println!("Deleting '{}'", secret.name);
        AwsSM::new("delete-secret")
            .args(["--secret-id", secret.arn.as_str()])
            .run();
    } else {
        println!("Aborting...")
    }
    Ok(())
}

/// Edits an existing secret
pub fn edit_secret(search_string: &str) -> Result<()> {
    let secret = select_secret(search_string)?;
    let secret_file = Temp::new_file()?;
    let secret_file_path = format!(
        "file://{}",
        &secret_file.to_str().expect("temp file should have path")
    );

    let secret_value = get_secret_value(&secret.arn, false)?;
    fs::write(secret_file.as_path(), &secret_value).expect("failed to write file");
    edit_file(secret_file.as_path());

    let new_contents = fs::read(secret_file.as_path()).unwrap();

    if new_contents == secret_value.into_bytes() {
        println!("Secret not changed. Aborting...")
    } else {
        let args = vec![
            "--secret-id",
            secret.arn.as_str(),
            "--secret-string",
            secret_file_path.as_str(),
        ];
        AwsSM::new("update-secret").args(args).run();

        println!("Updated secret {}", secret.name);
    }

    Ok(())
}
///
/// Describes an existing secret
pub fn describe_secret(search_string: &str) -> Result<()> {
    let secret = select_secret(search_string)?;
    let output = AwsSM::new("describe-secret")
        .args(["--secret-id", &secret.arn])
        .run();
    let secret_details = serde_json::from_str(&output)?;
    println!("{}", to_colored_json_auto(&secret_details).unwrap());
    Ok(())
}

/// Prints a table with a list of secrets
fn print_secret_table(secrets: &Vec<Secret>) {
    let table = Table::new(secrets)
        .with(Style::rounded())
        .with(Modify::new(Segment::all()).with(Alignment::left()));
    println!("\n{}\n", table);
}

/// Returns a list of secrets containing `search_string` in their name.
fn search_all_secrets(search_string: &str) -> Result<Vec<Secret>> {
    let output = AwsSM::new("list-secrets").run();
    let secret_list: SecretList = serde_json::from_str(&output)?;
    let mut secrets: Vec<Secret> = secret_list.list;

    secrets.retain(|s| {
        s.name
            .to_lowercase()
            .contains(&search_string.to_lowercase())
    });

    if secrets.is_empty() {
        eprintln!("There are no secrets matching \"{}\"", search_string);
        exit(1);
    }

    Ok(secrets)
}

/// Returns the secret value for a given ARN.
pub fn get_secret_value(arn: &str, colored: bool) -> Result<String> {
    let output = AwsSM::new("get-secret-value")
        .args(["--secret-id", arn])
        .run();
    let secret: Secret = serde_json::from_str(&output)?;

    // If the secret value is json serializable, parse it. Otherwise, return the string
    Ok(
        match serde_json::from_str::<serde_json::Value>(
            secret.value.as_ref().unwrap_or(&"".to_string()),
        ) {
            Ok(s) => {
                if colored {
                    to_colored_json_auto(&s)?
                } else {
                    to_string_pretty(&s)?
                }
            }
            Err(_) => secret.value.unwrap_or_else(|| "".to_string()),
        },
    )
}

/// Reads a usize integer from stdin.
fn read_int() -> Result<usize> {
    let mut input = String::new();

    stdout().flush()?;
    stdin().read_line(&mut input)?;
    Ok(input.trim().parse()?)
}

/// Reads a string from stdin.
fn read_string() -> Result<String> {
    let mut input = String::new();

    stdout().flush()?;
    stdin().read_line(&mut input)?;
    Ok(input.trim().parse()?)
}

// Opens a file in the user's preferred text editor.
fn edit_file(file: &Path) {
    let editor = get_editor();
    let status = std::process::Command::new(editor).arg(file).status();

    if status.is_err() {
        eprint!("Failed to open editor");
        exit(1)
    }
}

/// Gets the user's prefered text editor from `VISUAL` or `EDITOR` variables. Defaults to `vi` if
/// neither are set.
pub fn get_editor() -> String {
    if let Ok(r) = env::var("VISUAL") {
        return r;
    }
    if let Ok(r) = env::var("EDITOR") {
        return r;
    }
    "vi".to_string()
}
