use std::process::exit;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new secret
    #[clap(alias("c"))]
    #[clap(alias("create"))]
    Create {
        /// The name of the new secret
        secret_name: String,
        #[clap(short, long)]
        description: Option<String>,
    },
    /// Deletes an existing secret
    #[clap(alias("delete"))]
    Delete {
        /// The name of the secret delete
        secret_name: String,
    },
    /// Edits an existing secret
    #[clap(alias("e"))]
    #[clap(alias("edit"))]
    Edit {
        /// The name of the secret delete
        secret_name: String,
    },
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

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Create {
            secret_name,
            description,
        } => check_error(asm::create_secret(&secret_name, &description)),
        Commands::Delete { secret_name } => check_error(asm::delete_secret(&secret_name)),
        Commands::Edit { secret_name } => check_error(asm::edit_secret(&secret_name)),
        Commands::GetArn { search_string } => {
            let secret = check_error(asm::select_secret(&search_string));
            println!("{}", secret.arn);
        }
        Commands::GetValue { search_string } => {
            check_error(asm::search_and_get_value(&search_string))
        }
        Commands::Search { search_string } => check_error(asm::search_secret(&search_string)),
        Commands::List => check_error(asm::list_secrets()),
    };
}

fn check_error<T>(result: anyhow::Result<T>) -> T {
    match result {
        Ok(r) => r,
        Err(e) => {
            eprintln!("ERROR: {}", e);
            exit(1)
        }
    }
}
