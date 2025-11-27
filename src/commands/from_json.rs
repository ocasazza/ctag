use crate::api::ConfluenceClient;
use anyhow::{Context, Result};
use clap::Args;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;

#[derive(Args)]
pub struct FromJsonArgs {
    /// JSON file containing commands
    pub json_file: String,

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

/// Parse the `tags` value for add/remove actions as an array of strings.
pub(crate) fn parse_add_remove_tags(value: &Value, action: &str) -> Result<Vec<String>> {
    match value {
        Value::Array(items) => {
            let mut tags = Vec::with_capacity(items.len());
            for item in items {
                if let Some(s) = item.as_str() {
                    tags.push(s.to_string());
                } else {
                    anyhow::bail!(
                        "'tags' array for '{}' action must contain only strings",
                        action
                    );
                }
            }
            Ok(tags)
        }
        _ => anyhow::bail!(
            "'tags' field for '{}' action must be an array of strings",
            action
        ),
    }
}

/// Parse the `tags` value for replace actions as "old=new" pairs.
pub(crate) fn parse_replace_tag_pairs(value: &Value) -> Result<Vec<String>> {
    let map = match value {
        Value::Object(map) => map,
        _ => {
            anyhow::bail!(
                "'tags' field for 'replace' action must be an object mapping old->new tag"
            )
        }
    };

    let mut pairs = Vec::with_capacity(map.len());
    for (k, v) in map {
        if let Some(s) = v.as_str() {
            pairs.push(format!("{}={}", k, s));
        } else {
            anyhow::bail!("'tags' object for 'replace' action must map to string values");
        }
    }
    Ok(pairs)
}

pub fn run(
    args: FromJsonArgs,
    client: &ConfluenceClient,
    dry_run: bool,
    progress: bool,
) -> Result<()> {
    // Read and parse JSON file
    let json_content = fs::read_to_string(&args.json_file)
        .context(format!("Failed to read JSON file: {}", args.json_file))?;

    let json_commands: JsonCommands =
        serde_json::from_str(&json_content).context("Failed to parse JSON file")?;

    if let Some(desc) = &json_commands.description {
        println!("Description: {}", desc);
    }

    println!(
        "Found {} commands in the JSON file.",
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
                eprintln!("Unknown action: {}", command.action);
            }
        }
    }

    println!("\nAll commands completed.");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parse_add_remove_tags_valid_array() {
        let value = json!(["a", "b"]);
        let tags = parse_add_remove_tags(&value, "add").unwrap();
        assert_eq!(tags, vec!["a".to_string(), "b".to_string()]);
    }

    #[test]
    fn parse_add_remove_tags_rejects_non_array() {
        let value = json!({"a": "b"});
        assert!(parse_add_remove_tags(&value, "add").is_err());
    }

    #[test]
    fn parse_add_remove_tags_rejects_non_string_elements() {
        let value = json!(["a", 1]);
        assert!(parse_add_remove_tags(&value, "remove").is_err());
    }

    #[test]
    fn parse_replace_tag_pairs_valid_object() {
        let value = json!({"old": "new", "foo": "bar"});
        let mut pairs = parse_replace_tag_pairs(&value).unwrap();
        pairs.sort();
        assert_eq!(pairs, vec!["foo=bar".to_string(), "old=new".to_string()]);
    }

    #[test]
    fn parse_replace_tag_pairs_rejects_non_object() {
        let value = json!(["a", "b"]);
        assert!(parse_replace_tag_pairs(&value).is_err());
    }

    #[test]
    fn parse_replace_tag_pairs_rejects_non_string_values() {
        let value = json!({"a": 1});
        assert!(parse_replace_tag_pairs(&value).is_err());
    }
}
