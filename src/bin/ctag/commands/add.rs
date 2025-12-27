use crate::ui;
use anyhow::Result;
use clap::Args;
use colored::Colorize;
use ctag::api::ConfluenceClient;
use ctag::models::ProcessResults;
use dialoguer::Confirm;

#[derive(Args)]
#[command(after_help = "\
EXAMPLES:
  # Add tags to all pages in a space
  ctag add 'space = DOCS' tag1 tag2 tag3

  # Add tags to recently modified pages
  ctag add 'space = DOCS AND lastmodified > -7d' recent urgent

  # Preview changes before applying
  ctag --dry-run add 'space = DOCS' new-tag

  # Interactive mode with confirmation
  ctag add --interactive 'label = review' approved

")]
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
}

pub fn run(
    args: AddArgs,
    client: &ConfluenceClient,
    dry_run: bool,
    show_progress: bool,
    format: ctag::models::OutputFormat,
) -> Result<()> {
    let verbose = format.is_verbose();
    if verbose {
        ui::print_header("ADD TAGS");
    }
    // Get matching pages
    let pages = crate::commands::get_matching_pages(
        client,
        &args.cql_expression,
        100,
        format,
        show_progress,
    )?;

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
            let space = page.space_name();
            let display_title = page.printable_clickable_title(client.base_url());

            ui::print_page_action("Would add tags to", &display_title, space);
            for tag in &args.tags {
                ui::print_substep(&format!("{}: {}", "Add".green(), tag));
            }
        }
        return Ok(());
    }

    // Process the pages
    let mut results = ProcessResults::new(pages.len());
    if args.interactive {
        // Interactive mode: sequential processing
        let progress = if show_progress {
            Some(ui::create_progress_bar(pages.len() as u64))
        } else {
            None
        };
        for page in &pages {
            let page_id = match page.page_id() {
                Some(id) => id,
                None => {
                    results.skipped += 1;
                    continue;
                }
            };
            let space = page.space_name();
            let display_title = page.printable_clickable_title(client.base_url());
            if let Some(pb) = &progress {
                pb.suspend(|| {
                    ui::print_page_action("Adding tags to", &display_title, space);
                    for tag in &args.tags {
                        ui::print_substep(&format!("{}: {}", "Add".green(), tag));
                    }
                });
            } else {
                ui::print_page_action("Adding tags to", &display_title, space);
                for tag in &args.tags {
                    ui::print_substep(&format!("{}: {}", "Add".green(), tag));
                }
            }
            let prompt = format!(
                "Add tags {:?}? (Enter '{}' to abort)",
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
            let success = client.add_tags(page_id, &args.tags);
            results.processed += 1;
            if success {
                results.success += 1;
                results.tags_added += args.tags.len();
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
    } else {
        // Non-interactive mode: parallel processing
        results = crate::commands::process_pages_parallel(&pages, show_progress, |page| {
            let page_id = match page.page_id() {
                Some(id) => id,
                None => return crate::commands::ActionResult::Skipped,
            };
            if client.add_tags(page_id, &args.tags) {
                let detail = ctag::models::ActionDetail {
                    page_id: page_id.to_string(),
                    title: page.title.as_deref().unwrap_or("Unknown").to_string(),
                    space: page.space_name().to_string(),
                    url: page.printable_clickable_title(client.base_url()),
                    tags_added: args.tags.clone(),
                    tags_removed: vec![],
                };
                crate::commands::ActionResult::Success {
                    added: args.tags.len(),
                    removed: 0,
                    detail: Some(detail),
                }
            } else {
                crate::commands::ActionResult::Failed
            }
        });
    }

    // Display results
    ui::print_summary(&results, format);
    Ok(())
}
