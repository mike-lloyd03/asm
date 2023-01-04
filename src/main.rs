use clap::{Parser, Subcommand};

use asm::*;

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
            println!("{}", select_secret(&search_string).unwrap().arn);
        }
        Commands::GetValue { search_string } => search_and_get_value(search_string).unwrap(),
        Commands::Search { search_string } => search_cmd(search_string).unwrap(),
        Commands::List => list_cmd(),
    };
}
