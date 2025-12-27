pub mod add;
pub mod from_json;
pub mod from_stdin_json;
pub mod get;
pub mod remove;
pub mod replace;

use crate::ui;
use anyhow::Result;
use ctag::api::ConfluenceClient;
use ctag::models::{OutputFormat, SearchResultItem};

/// Shared logic to fetch pages with a spinner progress matching various settings
pub fn get_matching_pages(
    client: &ConfluenceClient,
    cql: &str,
    limit: usize,
    format: OutputFormat,
    show_progress: bool,
) -> Result<Vec<SearchResultItem>> {
    let verbose = format.is_verbose();
    let is_structured = format.is_structured();

    let spinner = if (verbose || !show_progress) && !is_structured {
        Some(ui::create_pagination_spinner(&format!(
            "Finding pages matching: {}",
            cql
        )))
    } else {
        None
    };

    let pages = if let Some(ref pb) = spinner {
        client.get_all_cql_results_with_progress(
            cql,
            limit,
            Some(|count, _| {
                pb.set_position(count as u64);
            }),
        )?
    } else {
        client.get_all_cql_results(cql, limit)?
    };

    if let Some(s) = spinner {
        s.finish_and_clear();
    }

    Ok(pages)
}

pub enum ActionResult {
    Success {
        added: usize,
        removed: usize,
        detail: Option<ctag::models::ActionDetail>,
    },
    Failed,
    Skipped,
}

/// Shared logic for processing pages in parallel with progress bar
pub fn process_pages_parallel<F>(
    pages: &[SearchResultItem],
    show_progress: bool,
    action: F,
) -> ctag::models::ProcessResults
where
    F: Fn(&SearchResultItem) -> ActionResult + Sync + Send,
{
    use rayon::prelude::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    let progress = if show_progress {
        Some(ui::create_progress_bar(pages.len() as u64))
    } else {
        None
    };

    let success_count = AtomicUsize::new(0);
    let failed_count = AtomicUsize::new(0);
    let skipped_count = AtomicUsize::new(0);
    let added_count = AtomicUsize::new(0);
    let removed_count = AtomicUsize::new(0);
    let details = Mutex::new(Vec::new());

    pages.par_iter().for_each(|page| {
        match action(page) {
            ActionResult::Success {
                added,
                removed,
                detail,
            } => {
                success_count.fetch_add(1, Ordering::Relaxed);
                added_count.fetch_add(added, Ordering::Relaxed);
                removed_count.fetch_add(removed, Ordering::Relaxed);
                if let Some(d) = detail {
                    if let Ok(mut g) = details.lock() {
                        g.push(d);
                    }
                }
            }
            ActionResult::Failed => {
                failed_count.fetch_add(1, Ordering::Relaxed);
            }
            ActionResult::Skipped => {
                skipped_count.fetch_add(1, Ordering::Relaxed);
            }
        }

        if let Some(ref p) = progress {
            p.inc(1);
        }
    });

    if let Some(ref p) = progress {
        p.finish_with_message("Done");
    }

    ctag::models::ProcessResults {
        total: pages.len(),
        processed: pages.len(),
        skipped: skipped_count.load(Ordering::Relaxed),
        success: success_count.load(Ordering::Relaxed),
        failed: failed_count.load(Ordering::Relaxed),
        aborted: false,
        tags_added: added_count.load(Ordering::Relaxed),
        tags_removed: removed_count.load(Ordering::Relaxed),
        details: details.into_inner().unwrap_or_default(),
    }
}
