use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use std::env;

mod api;
mod commands;
mod models;

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
    // Check environment variables
    let url = env::var("ATLASSIAN_URL").context("ATLASSIAN_URL must be set")?;
    let username = env::var("ATLASSIAN_USERNAME").context("ATLASSIAN_USERNAME must be set")?;
    let token = env::var("ATLASSIAN_TOKEN").context("ATLASSIAN_TOKEN must be set")?;
    let client = api::ConfluenceClient::new(url, username, token);
    match cli.command {
        Commands::Add(args) => commands::add::run(args, &client, cli.dry_run, cli.progress)?,
        Commands::Remove(args) => commands::remove::run(args, &client, cli.dry_run, cli.progress)?,
        Commands::Replace(args) => {
            commands::replace::run(args, &client, cli.dry_run, cli.progress)?
        }
        Commands::FromJson(args) => {
            commands::from_json::run(args, &client, cli.dry_run, cli.progress)?
        }
        Commands::FromStdinJson(args) => {
            commands::from_stdin_json::run(args, &client, cli.dry_run, cli.progress)?
        }
        Commands::Get(args) => commands::get::run(args, &client, cli.dry_run, cli.progress)?,
    }

    Ok(())
}
