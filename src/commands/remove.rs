use crate::api::{filter_excluded_pages, sanitize_text, ConfluenceClient};
use crate::models::ProcessResults;
use anyhow::Result;
use clap::Args;
use dialoguer::Confirm;
use indicatif::{ProgressBar, ProgressStyle};

#[derive(Args)]
pub struct RemoveArgs {
    /// CQL expression to match pages
    pub cql_expression: String,

    /// Tags to remove
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
    args: RemoveArgs,
    client: &ConfluenceClient,
    dry_run: bool,
    show_progress: bool,
) -> Result<()> {
    // Get matching pages
    eprintln!("Finding pages matching: {}", args.cql_expression);
    // TODO: This really needs to pagenate and get all results
    // ... it might be good to put rate-limiting and retry logic within this logic as well
    let mut pages = client.get_all_cql_results(&args.cql_expression, 100)?;
    if pages.is_empty() {
        eprintln!("No pages found matching the CQL expression.");
        if dry_run {
            eprintln!("DRY RUN: No changes will be made.");
        }
        return Ok(());
    }

    eprintln!("Found {} matching pages.", pages.len());

    // Apply exclusion if specified
    // TODO: remove exclusion filter from ctag entirely, other commands need it removed too
    if let Some(cql_exclude) = &args.cql_exclude {
        eprintln!("Finding pages to exclude: {}", cql_exclude);
        let excluded_pages = client.get_all_cql_results(cql_exclude, 100)?;
        if !excluded_pages.is_empty() {
            let original_count = pages.len();
            pages = filter_excluded_pages(pages, &excluded_pages);
            eprintln!(
                "Excluded {} pages. {} pages remaining.",
                original_count - pages.len(),
                pages.len()
            );
        }
    }

    if dry_run {
        eprintln!("DRY RUN: No changes will be made.");
        for page in &pages {
            let title = page.title.as_deref().unwrap_or("Unknown");
            let space = page
                .result_global_container
                .as_ref()
                .and_then(|c| c.title.as_deref())
                .unwrap_or("Unknown");
            eprintln!(
                "Would remove tags {:?} from '{}' (Space: {})",
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
                "Remove tags {:?} from {}? (Enter '{}' to abort)",
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
        let success = client.remove_tags(page_id, &args.tags);
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
    eprintln!("\nResults:");
    eprintln!("  Total pages: {}", results.total);
    eprintln!("  Processed: {}", results.processed);
    eprintln!("  Skipped: {}", results.skipped);
    eprintln!("  Successful: {}", results.success);
    eprintln!("  Failed: {}", results.failed);

    Ok(())
}
