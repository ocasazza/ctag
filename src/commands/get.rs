use crate::api::{filter_excluded_pages, sanitize_text, ConfluenceClient};
use anyhow::Result;
use clap::{Args, ValueEnum};
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

#[derive(Clone, ValueEnum)]
pub enum OutputFormat {
    Table,
    Json,
}

#[derive(Args)]
pub struct GetArgs {
    /// CQL expression to match pages
    pub cql_expression: String,

    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Table)]
    pub format: OutputFormat,

    /// Include page titles and spaces in output
    #[arg(long, default_value_t = true)]
    pub show_pages: bool,

    /// Show only unique tags across all pages
    #[arg(long)]
    pub tags_only: bool,

    /// Browse results interactively
    #[arg(long)]
    pub interactive: bool,

    /// Key to abort all operations in interactive mode
    #[arg(long, default_value = "q")]
    pub abort_key: String,

    /// CQL expression to match pages that should be excluded
    #[arg(long)]
    pub cql_exclude: Option<String>,

    /// Save results to file
    #[arg(long)]
    pub output_file: Option<String>,
}

#[derive(serde::Serialize)]
struct PageData {
    id: String,
    title: String,
    space: String,
    tags: Vec<String>,
}

pub fn run(
    args: GetArgs,
    client: &ConfluenceClient,
    _dry_run: bool,
    _show_progress: bool,
) -> Result<()> {
    // Get matching pages
    eprintln!("Finding pages matching: {}", args.cql_expression);
    let mut pages = client.get_all_cql_results(&args.cql_expression, 100)?;
    if pages.is_empty() {
        match args.format {
            OutputFormat::Json => println!("[]"),
            OutputFormat::Table => eprintln!("No pages found matching the CQL expression."),
        }
        return Ok(());
    }
    eprintln!("Found {} matching pages.", pages.len());
    // Apply exclusion if specified
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

    // Collect page data with tags
    eprintln!("Retrieving tags for pages...");
    let mut page_data = Vec::new();
    let mut all_tags = HashSet::new();
    for page in &pages {
        let page_id = match &page.content {
            Some(content) => match &content.id {
                Some(id) => id.clone(),
                None => {
                    eprintln!("Warning: Skipping page with no ID");
                    continue;
                }
            },
            None => {
                eprintln!("Warning: Skipping page with no content");
                continue;
            }
        };
        let title = sanitize_text(page.title.as_deref().unwrap_or("Unknown"));
        let space = page
            .result_global_container
            .as_ref()
            .and_then(|c| c.title.as_deref())
            .unwrap_or("Unknown")
            .to_string();

        let tags = client.get_page_tags(&page_id).unwrap_or_default();
        all_tags.extend(tags.iter().cloned());
        page_data.push(PageData {
            id: page_id,
            title,
            space,
            tags,
        });
    }
    // Generate output
    let output_content = if args.tags_only {
        format_tags_only(&all_tags, &args.format)
    } else {
        format_page_data(&page_data, &args.format, args.show_pages)
    };
    // Output results
    if let Some(output_file) = &args.output_file {
        let mut file = File::create(output_file)?;
        file.write_all(output_content.as_bytes())?;
        eprintln!("Results saved to {}", output_file);
    } else {
        println!("{}", output_content);
    }
    // Display summary
    eprintln!("\nSummary:");
    eprintln!("  Total pages processed: {}", page_data.len());
    eprintln!("  Unique tags found: {}", all_tags.len());
    Ok(())
}

fn format_tags_only(tags: &HashSet<String>, format: &OutputFormat) -> String {
    let mut sorted_tags: Vec<_> = tags.iter().collect();
    sorted_tags.sort();
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(&sorted_tags).unwrap_or_default(),
        OutputFormat::Table => {
            if sorted_tags.is_empty() {
                "No tags found.".to_string()
            } else {
                let mut output = String::from("Tags found:\n");
                output.push_str(&"=".repeat(50));
                output.push('\n');
                for tag in sorted_tags {
                    output.push_str(&format!("  {}\n", tag));
                }
                output
            }
        }
    }
}

fn format_page_data(page_data: &[PageData], format: &OutputFormat, show_pages: bool) -> String {
    match format {
        OutputFormat::Json => {
            if show_pages {
                serde_json::to_string_pretty(&page_data).unwrap_or_default()
            } else {
                let mut all_tags: HashSet<String> = HashSet::new();
                for page in page_data {
                    all_tags.extend(page.tags.iter().cloned());
                }
                let mut sorted: Vec<_> = all_tags.into_iter().collect();
                sorted.sort();
                serde_json::to_string_pretty(&sorted).unwrap_or_default()
            }
        }
        OutputFormat::Table => {
            if page_data.is_empty() {
                return "No pages found.".to_string();
            }
            let mut output = String::new();
            if show_pages {
                // Calculate column widths
                let max_title_len = page_data.iter().map(|p| p.title.len()).max().unwrap_or(0);
                let max_space_len = page_data.iter().map(|p| p.space.len()).max().unwrap_or(0);
                let title_width = max_title_len.min(50);
                let space_width = max_space_len.min(20);
                // Header
                output.push_str(&format!(
                    "{:<title_width$} {:<space_width$} Tags\n",
                    "Title",
                    "Space",
                    title_width = title_width,
                    space_width = space_width
                ));
                output.push_str(&"=".repeat(title_width + space_width + 20));
                output.push('\n');

                // Data rows
                for page in page_data {
                    let title = if page.title.len() > title_width {
                        &page.title[..title_width]
                    } else {
                        &page.title
                    };
                    let space = if page.space.len() > space_width {
                        &page.space[..space_width]
                    } else {
                        &page.space
                    };
                    let tags = if page.tags.is_empty() {
                        "(no tags)".to_string()
                    } else {
                        page.tags.join(", ")
                    };

                    output.push_str(&format!(
                        "{:<title_width$} {:<space_width$} {}\n",
                        title,
                        space,
                        tags,
                        title_width = title_width,
                        space_width = space_width
                    ));
                }
            } else {
                // Show only unique tags
                let mut all_tags: HashSet<String> = HashSet::new();
                for page in page_data {
                    all_tags.extend(page.tags.iter().cloned());
                }

                if all_tags.is_empty() {
                    output.push_str("No tags found.");
                } else {
                    output.push_str("Tags found:\n");
                    output.push_str(&"=".repeat(50));
                    output.push('\n');
                    let mut sorted: Vec<_> = all_tags.into_iter().collect();
                    sorted.sort();
                    for tag in sorted {
                        output.push_str(&format!("  {}\n", tag));
                    }
                }
            }

            output
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn format_tags_only_table_empty() {
        let tags: HashSet<String> = HashSet::new();
        let out = format_tags_only(&tags, &OutputFormat::Table);
        assert_eq!(out.trim(), "No tags found.");
    }

    #[test]
    fn format_tags_only_json_sorted() {
        let mut tags: HashSet<String> = HashSet::new();
        tags.insert("b".to_string());
        tags.insert("a".to_string());
        let out = format_tags_only(&tags, &OutputFormat::Json);
        let parsed: Vec<String> = serde_json::from_str(&out).unwrap();
        assert_eq!(parsed, vec!["a".to_string(), "b".to_string()]);
    }
    #[test]
    fn format_page_data_json_unique_tags_when_not_showing_pages() {
        let page_data = vec![
            PageData {
                id: "1".into(),
                title: "T1".into(),
                space: "S1".into(),
                tags: vec!["x".into(), "y".into()],
            },
            PageData {
                id: "2".into(),
                title: "T2".into(),
                space: "S1".into(),
                tags: vec!["y".into(), "z".into()],
            },
        ];
        let out = format_page_data(&page_data, &OutputFormat::Json, false);
        let tags: Vec<String> = serde_json::from_str(&out).unwrap();
        assert_eq!(
            tags,
            vec!["x".to_string(), "y".to_string(), "z".to_string()]
        );
    }

    #[test]
    fn format_page_data_table_shows_no_pages_message_when_empty() {
        let page_data: Vec<PageData> = Vec::new();
        let out = format_page_data(&page_data, &OutputFormat::Table, true);
        assert_eq!(out.trim(), "No pages found.");
    }
}
