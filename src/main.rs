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
