use crate::api::ConfluenceClient;
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
        println!("Description: {}", desc);
    }

    println!(
        "Found {} commands in the JSON data.",
        json_commands.commands.len()
    );

    for (i, command) in json_commands.commands.iter().enumerate() {
        println!(
            "\nExecuting command {}/{}: {} on {}",
            i + 1,
            json_commands.commands.len(),
            command.action,
            command.cql_expression
        );

        match command.action.as_str() {
            "add" => {
                let tags_value = command
                    .tags
                    .as_ref()
                    .context("'tags' field required for 'add' action")?;
                let tags = match tags_value {
                    Value::Array(items) => {
                        let mut tags = Vec::with_capacity(items.len());
                        for item in items {
                            if let Some(s) = item.as_str() {
                                tags.push(s.to_string());
                            } else {
                                anyhow::bail!(
                                    "'tags' array for 'add' action must contain only strings"
                                );
                            }
                        }
                        tags
                    }
                    _ => anyhow::bail!("'tags' field for 'add' action must be an array of strings"),
                };

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
                let tags = match tags_value {
                    Value::Array(items) => {
                        let mut tags = Vec::with_capacity(items.len());
                        for item in items {
                            if let Some(s) = item.as_str() {
                                tags.push(s.to_string());
                            } else {
                                anyhow::bail!(
                                    "'tags' array for 'remove' action must contain only strings"
                                );
                            }
                        }
                        tags
                    }
                    _ => anyhow::bail!(
                        "'tags' field for 'remove' action must be an array of strings"
                    ),
                };

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
                let tag_mapping: std::collections::HashMap<String, String> = match tags_value {
                    Value::Object(map) => {
                        let mut out = std::collections::HashMap::new();
                        for (k, v) in map {
                            if let Some(s) = v.as_str() {
                                out.insert(k.clone(), s.to_string());
                            } else {
                                anyhow::bail!(
                                    "'tags' object for 'replace' action must map to string values"
                                );
                            }
                        }
                        out
                    }
                    _ => anyhow::bail!(
                        "'tags' field for 'replace' action must be an object mapping old->new tag"
                    ),
                };
                let tag_pairs: Vec<String> = tag_mapping
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect();
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
                eprintln!("Unknown action: {}", command.action);
            }
        }
    }

    println!("\nAll commands completed.");
    Ok(())
}
