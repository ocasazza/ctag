use crate::api::{filter_excluded_pages, sanitize_text, ConfluenceClient};
use crate::models::ProcessResults;
use crate::ui;
use anyhow::Result;
use clap::Args;
use dialoguer::Confirm;

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
    format: crate::models::OutputFormat,
) -> Result<()> {
    let verbose = format == crate::models::OutputFormat::Verbose;
    let is_structured =
        format == crate::models::OutputFormat::Json || format == crate::models::OutputFormat::Csv;

    if verbose {
        ui::print_header("REMOVE TAGS");
    }

    // Get matching pages
    let spinner = if (verbose || !show_progress) && !is_structured {
        Some(ui::create_spinner(&format!(
            "Finding pages matching: {}",
            args.cql_expression
        )))
    } else {
        None
    };

    let mut pages = client.get_all_cql_results(&args.cql_expression, 100)?;

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

    // Apply exclusion if specified
    if let Some(cql_exclude) = &args.cql_exclude {
        let spinner = if (verbose || !show_progress) && !is_structured {
            Some(ui::create_spinner(&format!(
                "Finding pages to exclude: {}",
                cql_exclude
            )))
        } else {
            None
        };

        let excluded_pages = client.get_all_cql_results(cql_exclude, 100)?;

        if let Some(s) = spinner {
            s.finish_and_clear();
        }

        if !excluded_pages.is_empty() {
            let original_count = pages.len();
            pages = filter_excluded_pages(pages, &excluded_pages);
            if verbose {
                ui::print_info(&format!(
                    "Excluded {} pages. {} pages remaining.",
                    original_count - pages.len(),
                    pages.len()
                ));
            }
        }
    }

    if dry_run {
        ui::print_dry_run("No changes will be made.");
        for page in &pages {
            let title = page.title.as_deref().unwrap_or("Unknown");
            let space = page
                .result_global_container
                .as_ref()
                .and_then(|c| c.title.as_deref())
                .unwrap_or("Unknown");
            ui::print_page_action("Would remove tags", &sanitize_text(title), space);
            ui::print_substep(&format!("Tags: {:?}", args.tags));
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

        let title = page.title.as_deref().unwrap_or("Unknown");
        let space = page
            .result_global_container
            .as_ref()
            .and_then(|c| c.title.as_deref())
            .unwrap_or("Unknown");

        // Interactive confirmation
        if args.interactive {
            if let Some(pb) = &progress {
                pb.suspend(|| {
                    ui::print_page_action("Removing tags from", &sanitize_text(title), space);
                });
            } else {
                ui::print_page_action("Removing tags from", &sanitize_text(title), space);
            }

            let prompt = format!(
                "Remove tags {:?}? (Enter '{}' to abort)",
                args.tags, args.abort_key
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
    ui::print_summary(&results, format);

    Ok(())
}
