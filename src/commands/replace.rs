use crate::api::ConfluenceClient;
use crate::models::sanitize_text;
use crate::models::ProcessResults;
use crate::ui;
use anyhow::Result;
use clap::Args;
use colored::Colorize;
use dialoguer::Confirm;
use std::collections::HashMap;

#[derive(Args)]
#[command(after_help = "\
EXAMPLES:
  # Replace tags using old=new format
  ctag replace 'space = DOCS' 'old-tag=new-tag' 'foo=bar'

  # Replace tags with regex patterns (positional pairs)
  ctag replace --regex 'space = DOCS' 'test-.*' 'new-test' 'id-[0-9]+' 'matched-id'

  # Preview changes before applying
  ctag --dry-run replace 'space = DOCS' 'old=new'

  # Interactive mode with confirmation
  ctag replace --interactive 'space = DOCS' 'draft=published'

  # Multiple replacements with regex
  ctag replace --regex 'label = migration' \\
    'v1-.*' 'legacy' \\
    'temp-.*' 'archived'
")]
pub struct ReplaceArgs {
    /// CQL expression to match pages
    pub cql_expression: String,

    /// Tag pairs to replace
    /// - Without --regex: use 'old=new' format (e.g., 'foo=bar' 'baz=qux')
    /// - With --regex: use positional pairs (e.g., 'pattern1' 'replacement1' 'pattern2' 'replacement2')
    #[arg(required = true)]
    pub tag_pairs: Vec<String>,

    /// Confirm each action interactively
    #[arg(long)]
    pub interactive: bool,

    /// Key to abort all operations in interactive mode
    #[arg(long, default_value = "q")]
    pub abort_key: String,

    /// Use regex to match tags
    #[arg(long)]
    pub regex: bool,
}

/// Parse CLI tag pairs.
/// - If regex=false: expects ["old=new", "foo=bar"] format
/// - If regex=true: expects positional pairs ["old_regex", "new", "another_regex", "another_new"]
pub(crate) fn parse_tag_pairs(pairs: &[String], regex: bool) -> Result<HashMap<String, String>> {
    let mut tag_mapping = HashMap::new();

    if regex {
        // Positional pairs mode for regex
        if !pairs.len().is_multiple_of(2) {
            anyhow::bail!(
                "Invalid number of arguments for regex mode. Expected pairs of (old_pattern, new_tag), got {} arguments",
                pairs.len()
            );
        }

        for chunk in pairs.chunks(2) {
            let old = chunk[0].trim();
            let new = chunk[1].trim();

            if old.is_empty() || new.is_empty() {
                anyhow::bail!("Invalid tag pair: old pattern and new tag must be non-empty");
            }

            tag_mapping.insert(old.to_string(), new.to_string());
        }
    } else {
        // Traditional old=new format for non-regex mode
        for pair in pairs {
            let parts: Vec<&str> = pair.splitn(2, '=').collect();
            if parts.len() != 2 {
                anyhow::bail!(
                    "Invalid tag pair format: '{}'. Use format 'oldtag=newtag'",
                    pair
                );
            }
            let old = parts[0].trim();
            let new = parts[1].trim();

            if old.is_empty() || new.is_empty() {
                anyhow::bail!(
                    "Invalid tag pair format: '{}'. Old and new tags must be non-empty",
                    pair
                );
            }

            tag_mapping.insert(old.to_string(), new.to_string());
        }
    }

    Ok(tag_mapping)
}

pub fn run(
    args: ReplaceArgs,
    client: &ConfluenceClient,
    dry_run: bool,
    show_progress: bool,
    format: crate::models::OutputFormat,
) -> Result<()> {
    let verbose = format == crate::models::OutputFormat::Verbose;
    let is_structured =
        format == crate::models::OutputFormat::Json || format == crate::models::OutputFormat::Csv;

    if verbose {
        ui::print_header("REPLACE TAGS");
    }

    // Parse tag pairs
    let tag_mapping = parse_tag_pairs(&args.tag_pairs, args.regex)?;

    let compiled_regexes = if args.regex {
        let mut res = Vec::new();
        for (old, new) in &tag_mapping {
            res.push((
                regex::Regex::new(old)
                    .map_err(|e| anyhow::anyhow!("Invalid regex '{}': {}", old, e))?,
                new.clone(),
            ));
        }
        Some(res)
    } else {
        None
    };

    // Get matching pages
    let spinner = if (verbose || !show_progress) && !is_structured {
        Some(ui::create_spinner(&format!(
            "Finding pages matching: {}",
            args.cql_expression
        )))
    } else {
        None
    };

    let pages = client.get_all_cql_results(&args.cql_expression, 100)?;

    if let Some(s) = spinner {
        s.finish_and_clear();
    }

    if pages.is_empty() {
        ui::print_warning("No pages found matching the CQL expression.");
        if dry_run {
            ui::print_dry_run("No changes will be made.");
        }
        return Ok(());
    }
    if verbose {
        ui::print_info(&format!("Found {} matching pages.", pages.len()));
    }
    if dry_run {
        ui::print_dry_run("No changes will be made.");
        for page in &pages {
            let page_id = match &page.content {
                Some(content) => match &content.id {
                    Some(id) => id,
                    None => continue,
                },
                None => continue,
            };

            let title = page.title.as_deref().unwrap_or("Unknown");
            let space = page.space_name();

            let replacements = if let Some(regex_pairs) = &compiled_regexes {
                let current_tags = client.get_page_tags(page_id)?;
                crate::api::compute_replacements_by_regex(current_tags, regex_pairs)
            } else {
                tag_mapping.clone()
            };

            if replacements.is_empty() && args.regex {
                if verbose {
                    ui::print_info(&format!(
                        "Skipping page '{}' - no tags match regex",
                        sanitize_text(title)
                    ));
                }
                continue;
            }

            let display_title = page.printable_clickable_title(client.base_url());
            ui::print_page_action("Would replace tags on", &display_title, space);
            for (old, new) in &replacements {
                ui::print_substep(&format!(
                    "{}: {} {} {}",
                    "Replace".yellow(),
                    old.dimmed(),
                    "→".bright_black(),
                    new.green()
                ));
            }
        }
        return Ok(());
    }

    // Process the pages
    let mut results = ProcessResults::new(pages.len());
    let progress = if show_progress {
        Some(ui::create_progress_bar(pages.len() as u64))
    } else {
        None
    };

    for page in &pages {
        let page_id = match &page.content {
            Some(content) => match &content.id {
                Some(id) => id,
                None => {
                    results.skipped += 1;
                    continue;
                }
            },
            None => {
                results.skipped += 1;
                continue;
            }
        };

        let space = page.space_name();

        let replacements = if let Some(regex_pairs) = &compiled_regexes {
            let current_tags = client.get_page_tags(page_id)?;
            crate::api::compute_replacements_by_regex(current_tags, regex_pairs)
        } else {
            tag_mapping.clone()
        };

        if replacements.is_empty() && args.regex {
            results.skipped += 1;
            if let Some(pb) = &progress {
                pb.inc(1);
            }
            continue;
        }

        // Interactive confirmation
        if args.interactive {
            let display_title = page.printable_clickable_title(client.base_url());
            if let Some(pb) = &progress {
                pb.suspend(|| {
                    ui::print_page_action("Replacing tags on", &display_title, space);
                    for (old, new) in &replacements {
                        ui::print_substep(&format!(
                            "{}: {} {} {}",
                            "Replace".yellow(),
                            old.dimmed(),
                            "→".bright_black(),
                            new.green()
                        ));
                    }
                });
            } else {
                ui::print_page_action("Replacing tags on", &display_title, space);
                for (old, new) in &replacements {
                    ui::print_substep(&format!(
                        "{}: {} {} {}",
                        "Replace".yellow(),
                        old.dimmed(),
                        "→".bright_black(),
                        new.green()
                    ));
                }
            }

            let old_tags: Vec<_> = replacements.keys().collect();
            let new_tags: Vec<_> = replacements.values().collect();
            let prompt = format!(
                "Replace tags {:?} with {:?}? (Enter '{}' to abort)",
                old_tags, new_tags, args.abort_key
            );

            let confirmed = if let Some(pb) = &progress {
                pb.suspend(|| Confirm::new().with_prompt(&prompt).interact())
            } else {
                Confirm::new().with_prompt(&prompt).interact()
            };

            match confirmed {
                Ok(true) => {}
                Ok(false) => {
                    results.skipped += 1;
                    if let Some(pb) = &progress {
                        pb.inc(1);
                    }
                    continue;
                }
                Err(_) => {
                    results.aborted = true;
                    break;
                }
            }
        }

        // Perform the action
        let success = client.replace_tags(page_id, &replacements);
        results.processed += 1;

        if success {
            results.success += 1;
        } else {
            results.failed += 1;
        }

        if let Some(pb) = &progress {
            pb.inc(1);
        }
    }

    if let Some(pb) = progress {
        pb.finish_with_message("Done");
    }

    // Display results
    ui::print_summary(&results, format);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tag_pairs_trims_whitespace_and_parses_correctly() {
        let input = vec!["old=new".to_string(), " foo = bar ".to_string()];

        let mapping = parse_tag_pairs(&input, false).unwrap();
        assert_eq!(mapping.get("old"), Some(&"new".to_string()));
        assert_eq!(mapping.get("foo"), Some(&"bar".to_string()));
    }

    #[test]
    fn parse_tag_pairs_rejects_missing_equal_sign() {
        let input = vec!["invalidpair".to_string()];
        let err = parse_tag_pairs(&input, false).unwrap_err();
        let msg = format!("{}", err);
        assert!(
            msg.contains("Invalid tag pair format"),
            "unexpected error message: {}",
            msg
        );
    }

    #[test]
    fn parse_tag_pairs_rejects_empty_old_or_new() {
        let inputs = vec![
            "=new".to_string(),
            "old=".to_string(),
            " = new ".to_string(),
            " old =  ".to_string(),
        ];

        for s in inputs {
            let err = parse_tag_pairs(std::slice::from_ref(&s), false).unwrap_err();
            let msg = format!("{}", err);
            assert!(
                msg.contains("Old and new tags must be non-empty"),
                "unexpected error for '{}': {}",
                s,
                msg
            );
        }
    }

    #[test]
    fn parse_tag_pairs_positional_mode_works() {
        let input = vec![
            "test-.*".to_string(),
            "new-test".to_string(),
            "id-[0-9]+".to_string(),
            "matched-id".to_string(),
        ];

        let mapping = parse_tag_pairs(&input, true).unwrap();
        assert_eq!(mapping.get("test-.*"), Some(&"new-test".to_string()));
        assert_eq!(mapping.get("id-[0-9]+"), Some(&"matched-id".to_string()));
    }

    #[test]
    fn parse_tag_pairs_positional_mode_rejects_odd_count() {
        let input = vec![
            "test-.*".to_string(),
            "new-test".to_string(),
            "orphan".to_string(),
        ];

        let err = parse_tag_pairs(&input, true).unwrap_err();
        let msg = format!("{}", err);
        assert!(
            msg.contains("Invalid number of arguments"),
            "unexpected error message: {}",
            msg
        );
    }

    #[test]
    fn parse_tag_pairs_positional_mode_rejects_empty() {
        let inputs = vec![
            vec!["".to_string(), "new".to_string()],
            vec!["old".to_string(), "".to_string()],
        ];

        for input in inputs {
            let err = parse_tag_pairs(&input, true).unwrap_err();
            let msg = format!("{}", err);
            assert!(
                msg.contains("must be non-empty"),
                "unexpected error message: {}",
                msg
            );
        }
    }
}
