use crate::api::ConfluenceClient;
use crate::commands::from_json::{process_single_command, JsonCommands};
use crate::models::{OutputFormat, ProcessResults};
use crate::ui;
use anyhow::{Context, Result};
use clap::Args;
use std::io::{self, Read};

#[derive(Args)]
pub struct FromStdinJsonArgs {
    /// Key to abort all operations in interactive mode
    #[arg(long, default_value = "q")]
    pub abort_key: String,
}

pub fn run(
    args: FromStdinJsonArgs,
    client: &ConfluenceClient,
    dry_run: bool,
    progress: bool,
    format: OutputFormat,
) -> Result<()> {
    let verbose = format == OutputFormat::Verbose;
    let is_structured = format == OutputFormat::Json || format == OutputFormat::Csv;

    if verbose {
        ui::print_header("EXECUTE FROM STDIN JSON");
    }

    // Read from stdin
    let mut buffer = String::new();
    io::stdin()
        .read_to_string(&mut buffer)
        .context("Failed to read from stdin")?;

    if buffer.trim().is_empty() {
        anyhow::bail!("No data provided via stdin. Use a pipe or redirect to provide JSON data.");
    }

    // Parse JSON
    let json_commands: JsonCommands =
        serde_json::from_str(&buffer).context("Failed to parse JSON from stdin")?;

    if let Some(desc) = &json_commands.description {
        if verbose {
            ui::print_info(&format!("Description: {}", desc));
        }
    }

    if verbose {
        ui::print_info(&format!(
            "Found {} commands in the JSON data.",
            json_commands.commands.len()
        ));
    }

    let mut results = ProcessResults::new(json_commands.commands.len());

    for (i, command) in json_commands.commands.iter().enumerate() {
        if verbose {
            ui::print_step(&format!(
                "Command {}/{}: {} on {}",
                i + 1,
                json_commands.commands.len(),
                command.action.to_uppercase(),
                command.cql_expression
            ));
        }

        match process_single_command(command, client, dry_run, progress, format, &args.abort_key) {
            Ok(_) => {
                results.processed += 1;
                results.success += 1;
            }
            Err(e) => {
                results.processed += 1;
                results.failed += 1;
                if verbose || !is_structured {
                    ui::print_error(&format!("Command failed: {}", e));
                }
            }
        }
    }

    ui::print_summary(&results, format);
    Ok(())
}
