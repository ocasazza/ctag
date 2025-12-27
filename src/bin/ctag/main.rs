use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use std::env;

// commands and ui handle CLI interaction, so they stay in bin for now.
// Eventually commands content should move to lib::ops, leaving only CLI parsing here.
mod commands;
mod ui;

use ctag::api;
use ctag::models::OutputFormat;

#[derive(Parser)]
#[command(name = "ctag")]
#[command(about = "ctag - Manage Confluence page tags in bulk with a CLI.", long_about = None)]
#[command(version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Show progress bars during operations
    #[arg(long, default_value_t = true, global = true)]
    progress: bool,

    /// Preview changes without making any modifications
    #[arg(long, global = true)]
    dry_run: bool,

    /// Output format
    #[arg(long, value_enum, global = true)]
    format: Option<OutputFormat>,

    /// Show detailed output (shortcut for --format verbose)
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    Add(commands::add::AddArgs),
    Remove(commands::remove::RemoveArgs),
    Replace(commands::replace::ReplaceArgs),
    #[command(name = "from-json")]
    FromJson(commands::from_json::FromJsonArgs),
    #[command(name = "from-stdin-json")]
    FromStdinJson(commands::from_stdin_json::FromStdinJsonArgs),
    Get(commands::get::GetArgs),
}

fn main() -> Result<()> {
    dotenv().ok();
    env_logger::init();
    let cli = Cli::parse();

    // Determine the output format
    let format = if let Some(f) = cli.format {
        f
    } else if cli.verbose {
        OutputFormat::Verbose
    } else {
        OutputFormat::Simple
    };

    // Check environment variables
    let url = env::var("ATLASSIAN_URL").context("ATLASSIAN_URL must be set")?;
    let username = env::var("ATLASSIAN_USERNAME").context("ATLASSIAN_USERNAME must be set")?;
    let token = env::var("ATLASSIAN_TOKEN").context("ATLASSIAN_TOKEN must be set")?;
    let client = api::ConfluenceClient::new(url, username, token);

    match cli.command {
        Commands::Add(args) => {
            commands::add::run(args, &client, cli.dry_run, cli.progress, format)?
        }
        Commands::Remove(args) => {
            commands::remove::run(args, &client, cli.dry_run, cli.progress, format)?
        }
        Commands::Replace(args) => {
            commands::replace::run(args, &client, cli.dry_run, cli.progress, format)?
        }
        Commands::FromJson(args) => {
            commands::from_json::run(args, &client, cli.dry_run, cli.progress, format)?
        }
        Commands::FromStdinJson(args) => {
            commands::from_stdin_json::run(args, &client, cli.dry_run, cli.progress, format)?
        }
        Commands::Get(args) => commands::get::run(args, &client, cli.progress, format)?,
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli_version_matches_cargo() {
        let cmd = Cli::command();
        let version = cmd
            .get_version()
            .expect("Version should be set on CLI command");
        let cargo_version = env!("CARGO_PKG_VERSION");
        assert_eq!(
            version, cargo_version,
            "CLI version ({}) does not match Cargo.toml version ({})",
            version, cargo_version
        );
    }
}
