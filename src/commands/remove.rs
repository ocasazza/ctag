use crate::api::{filter_excluded_pages, ConfluenceClient};
use crate::models::sanitize_text;
use crate::models::ProcessResults;
use crate::ui;
use anyhow::Result;
use clap::Args;
use colored::Colorize;
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

    /// Use regex to match tags
    #[arg(long)]
    pub regex: bool,
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

    let compiled_regexes = if args.regex {
        let mut res = Vec::new();
        for t in &args.tags {
            res.push(
                regex::Regex::new(t)
                    .map_err(|e| anyhow::anyhow!("Invalid regex '{}': {}", t, e))?,
            );
        }
        Some(res)
    } else {
        None
    };

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
            let page_id = match &page.content {
                Some(content) => match &content.id {
                    Some(id) => id,
                    None => continue,
                },
                None => continue,
            };

            let title = page.title.as_deref().unwrap_or("Unknown");
            let space = page.space_name();

            let tags_to_remove = if let Some(regexes) = &compiled_regexes {
                let current_tags = client.get_page_tags(page_id)?;
                crate::api::filter_tags_by_regex(current_tags, regexes)
            } else {
                args.tags.clone()
            };

            if tags_to_remove.is_empty() && args.regex {
                if verbose {
                    ui::print_info(&format!(
                        "Skipping page '{}' - no tags match regex",
                        sanitize_text(title)
                    ));
                }
                continue;
            }

            let display_title = page.printable_clickable_title(client.base_url());
            ui::print_page_action("Would remove tags from", &display_title, space);
            for tag in &tags_to_remove {
                ui::print_substep(&format!("{}: {}", "Remove".red(), tag));
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

        let tags_to_remove = if let Some(regexes) = &compiled_regexes {
            let current_tags = client.get_page_tags(page_id)?;
            crate::api::filter_tags_by_regex(current_tags, regexes)
        } else {
            args.tags.clone()
        };

        if tags_to_remove.is_empty() && args.regex {
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
                    ui::print_page_action("Removing tags from", &display_title, space);
                    for tag in &tags_to_remove {
                        ui::print_substep(&format!("{}: {}", "Remove".red(), tag));
                    }
                });
            } else {
                ui::print_page_action("Removing tags from", &display_title, space);
                for tag in &tags_to_remove {
                    ui::print_substep(&format!("{}: {}", "Remove".red(), tag));
                }
            }

            let prompt = format!(
                "Remove tags {:?}? (Enter '{}' to abort)",
                tags_to_remove, args.abort_key
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
        let success = client.remove_tags(page_id, &tags_to_remove);
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
