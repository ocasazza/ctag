use crate::api::{filter_excluded_pages, sanitize_text, ConfluenceClient};
use crate::models::ProcessResults;
use anyhow::Result;
use clap::Args;
use dialoguer::Confirm;
use indicatif::{ProgressBar, ProgressStyle};
use std::collections::HashMap;

#[derive(Args)]
pub struct ReplaceArgs {
    /// CQL expression to match pages
    pub cql_expression: String,

    /// Tag pairs (old=new)
    #[arg(required = true)]
    pub tag_pairs: Vec<String>,

    /// Confirm each action interactively
    #[arg(long)]
    pub interactive: bool,

    /// Key to abort all operations in interactive mode
    #[arg(long, default_value = "q")]
    pub abort_key: String,

    /// CQL expression to match pages that should be excluded
    #[arg(long)]
    pub cql_exclude: Option<String>,
}

/// Parse CLI tag pairs like ["old=new", "foo=bar"] into a mapping.
pub(crate) fn parse_tag_pairs(pairs: &[String]) -> Result<HashMap<String, String>> {
    let mut tag_mapping = HashMap::new();

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

    Ok(tag_mapping)
}

pub fn run(
    args: ReplaceArgs,
    client: &ConfluenceClient,
    dry_run: bool,
    show_progress: bool,
) -> Result<()> {
    // Parse tag pairs
    let tag_mapping = parse_tag_pairs(&args.tag_pairs)?;

    // Get matching pages
    println!("Finding pages matching: {}", args.cql_expression);
    let mut pages = client.get_all_cql_results(&args.cql_expression, 100)?;

    if pages.is_empty() {
        println!("No pages found matching the CQL expression.");
        if dry_run {
            println!("DRY RUN: No changes will be made.");
        }
        return Ok(());
    }

    println!("Found {} matching pages.", pages.len());

    // Apply exclusion if specified
    if let Some(cql_exclude) = &args.cql_exclude {
        println!("Finding pages to exclude: {}", cql_exclude);
        let excluded_pages = client.get_all_cql_results(cql_exclude, 100)?;
        if !excluded_pages.is_empty() {
            let original_count = pages.len();
            pages = filter_excluded_pages(pages, &excluded_pages);
            println!(
                "Excluded {} pages. {} pages remaining.",
                original_count - pages.len(),
                pages.len()
            );
        }
    }

    if dry_run {
        println!("DRY RUN: No changes will be made.");
        for page in &pages {
            let title = page.title.as_deref().unwrap_or("Unknown");
            let space = page
                .result_global_container
                .as_ref()
                .and_then(|c| c.title.as_deref())
                .unwrap_or("Unknown");
            let old_tags: Vec<_> = tag_mapping.keys().collect();
            let new_tags: Vec<_> = tag_mapping.values().collect();
            println!(
                "Would replace tags {:?} with {:?} on '{}' (Space: {})",
                old_tags,
                new_tags,
                sanitize_text(title),
                space
            );
        }
        return Ok(());
    }

    // Process the pages
    let mut results = ProcessResults::new(pages.len());
    let progress = if show_progress {
        let pb = ProgressBar::new(pages.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("[{elapsed_precise}] {bar:40.cyan/blue} {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("##-"),
        );
        Some(pb)
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

        let title = page.title.as_deref().unwrap_or("Unknown");
        let space = page
            .result_global_container
            .as_ref()
            .and_then(|c| c.title.as_deref())
            .unwrap_or("Unknown");

        // Interactive confirmation
        if args.interactive {
            let page_info = format!(
                "'{}' (Space: {}, ID: {})",
                sanitize_text(title),
                space,
                page_id
            );
            let old_tags: Vec<_> = tag_mapping.keys().collect();
            let new_tags: Vec<_> = tag_mapping.values().collect();
            let prompt = format!(
                "Replace tags {:?} with {:?} on {}? (Enter '{}' to abort)",
                old_tags, new_tags, page_info, args.abort_key
            );

            match Confirm::new().with_prompt(&prompt).interact() {
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
        let success = client.replace_tags(page_id, &tag_mapping);
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
    println!("\nResults:");
    println!("  Total pages: {}", results.total);
    println!("  Processed: {}", results.processed);
    println!("  Skipped: {}", results.skipped);
    println!("  Successful: {}", results.success);
    println!("  Failed: {}", results.failed);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_tag_pairs_trims_whitespace_and_parses_correctly() {
        let input = vec!["old=new".to_string(), " foo = bar ".to_string()];

        let mapping = parse_tag_pairs(&input).unwrap();
        assert_eq!(mapping.get("old"), Some(&"new".to_string()));
        assert_eq!(mapping.get("foo"), Some(&"bar".to_string()));
    }

    #[test]
    fn parse_tag_pairs_rejects_missing_equal_sign() {
        let input = vec!["invalidpair".to_string()];
        let err = parse_tag_pairs(&input).unwrap_err();
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
            let err = parse_tag_pairs(std::slice::from_ref(&s)).unwrap_err();
            let msg = format!("{}", err);
            assert!(
                msg.contains("Old and new tags must be non-empty"),
                "unexpected error for '{}': {}",
                s,
                msg
            );
        }
    }
}
