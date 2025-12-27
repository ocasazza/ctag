use crate::api::ConfluenceClient;
use crate::models::{ProcessResults, SearchResultItem};
use anyhow::Result;

pub struct OpsOptions {
    pub show_progress: bool,
}

pub trait ProgressReporter: Send + Sync {
    fn set_total(&self, total: u64);
    fn inc(&self, delta: u64);
    fn finish(&self);
    fn message(&self, msg: &str);
}

/// No-op progress reporter for when progress is disabled
pub struct NoOpProgress;
impl ProgressReporter for NoOpProgress {
    fn set_total(&self, _total: u64) {}
    fn inc(&self, _delta: u64) {}
    fn finish(&self) {}
    fn message(&self, _msg: &str) {}
}

pub enum ActionResult {
    Success {
        added: usize,
        removed: usize,
        detail: Option<crate::models::ActionDetail>,
    },
    Failed,
    Skipped,
}

/// Helper to run action on pages in parallel
pub fn process_pages_parallel<F>(
    pages: &[SearchResultItem],
    progress_reporter: Option<&dyn ProgressReporter>,
    action: F,
) -> ProcessResults
where
    F: Fn(&SearchResultItem) -> ActionResult + Sync + Send,
{
    use rayon::prelude::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Mutex;

    let reporter = progress_reporter.unwrap_or(&NoOpProgress);
    reporter.set_total(pages.len() as u64);

    let success_count = AtomicUsize::new(0);
    let failed_count = AtomicUsize::new(0);
    let skipped_count = AtomicUsize::new(0);
    let added_count = AtomicUsize::new(0);
    let removed_count = AtomicUsize::new(0);

    // We need to collect details safely across threads
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
        reporter.inc(1);
    });

    reporter.finish();

    ProcessResults {
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

pub fn get_matching_pages(
    client: &ConfluenceClient,
    cql: &str,
    limit: usize,
    progress_reporter: Option<&dyn ProgressReporter>,
) -> Result<Vec<SearchResultItem>> {
    if let Some(p) = progress_reporter {
        p.message(&format!("Finding pages matching: {}", cql));
        client.get_all_cql_results_with_progress(
            cql,
            limit,
            Some(|count, _| {
                p.set_total(count as u64 + 100); // Rough estimate fix or just set position
                                                 // Note: the original progress bar logic was "spinner", so "set_position" just spins
            }),
        )
    } else {
        client.get_all_cql_results(cql, limit)
    }
}
