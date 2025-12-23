use crate::api::ConfluenceClient;
use crate::commands::from_json::{parse_add_remove_tags, parse_replace_tag_pairs};
use crate::ui;
use anyhow::{Context, Result};
use clap::Args;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::io::{self, Read};

#[derive(Args)]
pub struct FromStdinJsonArgs {
    /// Key to abort all operations in interactive mode
    #[arg(long, default_value = "q")]
    pub abort_key: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct JsonCommands {
    description: Option<String>,
    commands: Vec<JsonCommand>,
}

#[derive(Debug, Deserialize, Serialize)]
struct JsonCommand {
    action: String,
    cql_expression: String,
    /// Tags field is overloaded to match the original Python implementation:
    /// - For "add" and "remove": array of strings, e.g. ["tag1", "tag2"]
    /// - For "replace": object mapping "old" -> "new", e.g. {"old-tag": "new-tag"}
    #[serde(default)]
    tags: Option<Value>,
    #[serde(default)]
    interactive: bool,
    cql_exclude: Option<String>,
}

pub fn run(
    args: FromStdinJsonArgs,
    client: &ConfluenceClient,
    dry_run: bool,
    progress: bool,
) -> Result<()> {
    ui::print_header("EXECUTE FROM STDIN JSON");

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
        ui::print_info(&format!("Description: {}", desc));
    }

    ui::print_info(&format!(
        "Found {} commands in the JSON data.",
        json_commands.commands.len()
    ));

    for (i, command) in json_commands.commands.iter().enumerate() {
        ui::print_step(&format!(
            "Command {}/{}: {} on {}",
            i + 1,
            json_commands.commands.len(),
            command.action.to_uppercase(),
            command.cql_expression
        ));

        match command.action.as_str() {
            "add" => {
                let tags_value = command
                    .tags
                    .as_ref()
                    .context("'tags' field required for 'add' action")?;
                let tags = parse_add_remove_tags(tags_value, "add")?;

                let add_args = crate::commands::add::AddArgs {
                    cql_expression: command.cql_expression.clone(),
                    tags,
                    interactive: command.interactive,
                    abort_key: args.abort_key.clone(),
                    cql_exclude: command.cql_exclude.clone(),
                };
                crate::commands::add::run(add_args, client, dry_run, progress)?;
            }
            "remove" => {
                let tags_value = command
                    .tags
                    .as_ref()
                    .context("'tags' field required for 'remove' action")?;
                let tags = parse_add_remove_tags(tags_value, "remove")?;

                let remove_args = crate::commands::remove::RemoveArgs {
                    cql_expression: command.cql_expression.clone(),
                    tags,
                    interactive: command.interactive,
                    abort_key: args.abort_key.clone(),
                    cql_exclude: command.cql_exclude.clone(),
                };
                crate::commands::remove::run(remove_args, client, dry_run, progress)?;
            }
            "replace" => {
                let tags_value = command
                    .tags
                    .as_ref()
                    .context("'tags' field required for 'replace' action")?;
                let tag_pairs = parse_replace_tag_pairs(tags_value)?;

                let replace_args = crate::commands::replace::ReplaceArgs {
                    cql_expression: command.cql_expression.clone(),
                    tag_pairs,
                    interactive: command.interactive,
                    abort_key: args.abort_key.clone(),
                    cql_exclude: command.cql_exclude.clone(),
                };
                crate::commands::replace::run(replace_args, client, dry_run, progress)?;
            }
            _ => {
                ui::print_error(&format!("Unknown action: {}", command.action));
            }
        }
    }

    ui::print_success("All commands completed.");
    Ok(())
}
