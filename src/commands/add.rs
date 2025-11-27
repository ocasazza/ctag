use crate::api::{filter_excluded_pages, sanitize_text, ConfluenceClient};
use crate::models::ProcessResults;
use anyhow::Result;
use clap::Args;
use dialoguer::Confirm;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Args)]
pub struct AddArgs {
    /// CQL expression to match pages
    pub cql_expression: String,

    /// Tags to add
    #[arg(required = true)]
    pub tags: Vec<String>,

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

pub fn run(
    args: AddArgs,
    client: &ConfluenceClient,
    dry_run: bool,
    show_progress: bool,
) -> Result<()> {
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
            println!(
                "Would add tags {:?} to '{}' (Space: {})",
                args.tags,
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
            let prompt = format!(
                "Add tags {:?} to {}? (Enter '{}' to abort)",
                args.tags, page_info, args.abort_key
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
        let success = client.add_tags(page_id, &args.tags);
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
