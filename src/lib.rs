use std::fs;
use std::io::{stdin, stdout, Write};
use std::{env, path::Path, process::exit};

use anyhow::Result;
use colored_json::to_colored_json_auto;
use dialoguer::theme::ColorfulTheme;
use dialoguer::FuzzySelect;
use serde_json::to_string_pretty;

mod aws_sm;
mod secret;

pub use aws_sm::AwsSM;
pub use secret::{Secret, SecretList};
use tempfile::{Builder, NamedTempFile};

/// Returns a secret from a list of secrets based on `search_string`. If more than one secret is
/// returned from the search, the user is prompted to choose one.
pub fn select_secret(search_string: &str) -> Result<Secret> {
    let mut secrets = search_all_secrets(search_string)?;
    let items: Vec<String> = secrets.list.iter().map(|s| s.name.clone()).collect();

    let i = if secrets.list.len() > 1 {
        FuzzySelect::with_theme(&ColorfulTheme::default())
            .with_prompt("Select secret")
            .items(&items)
            .interact()?
    } else {
        eprintln!(
            "{}",
            secrets.list.first().expect("should be one secret").name
        );
        0
    };

    Ok(secrets.list.remove(i))
}

/// Prints a list of secrets containing `search_string`
pub fn search_secret(search_string: &str) -> Result<()> {
    let secrets = search_all_secrets(search_string)?;

    secrets.print_table();
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
    secrets.print_table();
    Ok(())
}

/// Creates a new secret
pub fn create_secret(secret_name: &str, description: &Option<String>) -> Result<()> {
    let secret_file = create_temp_file(Some(".json"))?;
    let secret_file_path = format!(
        "file://{}",
        &secret_file
            .path()
            .to_str()
            .expect("temp file should have path")
    );

    edit_file(secret_file.path());

    if Path::exists(secret_file.path()) {
        let mut args = vec!["--name", secret_name, "--secret-string", &secret_file_path];
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
pub fn edit_secret(search_string: &str, edit_description: bool) -> Result<()> {
    let secret = select_secret(search_string)?;
    let secret_file = create_temp_file(Some(".json"))?;
    let secret_file_path = format!(
        "file://{}",
        &secret_file
            .path()
            .to_str()
            .expect("temp file should have path")
    );

    if !edit_description {
        let secret_value = get_secret_value(&secret.arn, false)?;
        fs::write(secret_file.path(), &secret_value).expect("failed to write file");
        edit_file(secret_file.path());

        let new_contents = fs::read(secret_file.path()).unwrap();

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
    } else {
        let desc = secret.description.unwrap_or_default();
        fs::write(secret_file.path(), desc.clone()).expect("failed to write file");
        edit_file(secret_file.path());

        let new_contents = fs::read(secret_file.path()).unwrap();

        if new_contents == desc.into_bytes() {
            println!("Description not changed. Aborting...")
        } else {
            let args = vec![
                "--secret-id",
                secret.arn.as_str(),
                "--description",
                secret_file_path.as_str(),
            ];
            AwsSM::new("update-secret").args(args).run();

            println!("Updated secret description {}", secret.name);
        }
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
    println!("{}", to_colored_json_auto(&secret_details)?);

    Ok(())
}

/// Returns a list of secrets containing `search_string` in their name.
fn search_all_secrets(search_string: &str) -> Result<SecretList> {
    let output = AwsSM::new("list-secrets").run();
    let mut secrets: SecretList = serde_json::from_str(&output)?;

    secrets.list.retain(|s| {
        s.name
            .to_lowercase()
            .contains(&search_string.to_lowercase())
    });

    if secrets.list.is_empty() {
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
            Err(_) => secret.value.unwrap_or_default(),
        },
    )
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

// Creates a temporary file with the optionally provided suffix.
fn create_temp_file(suffix: Option<&str>) -> Result<NamedTempFile> {
    let mut builder = Builder::new();
    if let Some(s) = suffix {
        builder.suffix(s);
    };
    Ok(builder.tempfile()?)
}
