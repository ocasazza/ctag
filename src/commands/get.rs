use crate::api::{filter_excluded_pages, sanitize_text, ConfluenceClient};
use crate::ui;
use anyhow::Result;
use clap::{Args, ValueEnum};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
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
    show_progress: bool,
) -> Result<()> {
    ui::print_header("GET TAGS");

    // Get matching pages
    ui::print_step(&format!("Finding pages matching: {}", args.cql_expression));
    let mut pages = client.get_all_cql_results(&args.cql_expression, 100)?;
    if pages.is_empty() {
        match args.format {
            OutputFormat::Json => println!("[]"),
            OutputFormat::Table => ui::print_warning("No pages found matching the CQL expression."),
        }
        return Ok(());
    }
    ui::print_info(&format!("Found {} matching pages.", pages.len()));

    // Apply exclusion if specified
    if let Some(cql_exclude) = &args.cql_exclude {
        ui::print_step(&format!("Finding pages to exclude: {}", cql_exclude));
        let excluded_pages = client.get_all_cql_results(cql_exclude, 100)?;
        if !excluded_pages.is_empty() {
            let original_count = pages.len();
            pages = filter_excluded_pages(pages, &excluded_pages);
            ui::print_info(&format!(
                "Excluded {} pages. {} pages remaining.",
                original_count - pages.len(),
                pages.len()
            ));
        }
    }

    // Collect page data with tags
    ui::print_step("Retrieving tags for pages...");
    let mut page_data = Vec::new();
    let mut all_tags = HashSet::new();

    let progress = if show_progress {
        Some(ui::create_progress_bar(pages.len() as u64))
    } else {
        None
    };

    for page in &pages {
        let page_id = match &page.content {
            Some(content) => match &content.id {
                Some(id) => id.clone(),
                None => {
                    ui::print_warning("Skipping page with no ID");
                    continue;
                }
            },
            None => {
                ui::print_warning("Skipping page with no content");
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

        if let Some(pb) = &progress {
            pb.inc(1);
        }
    }

    if let Some(pb) = progress {
        pb.finish_and_clear();
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
        ui::print_success(&format!("Results saved to {}", output_file));
    } else {
        println!("{}", output_content);
    }

    // Display summary
    ui::print_info(&format!("Total pages processed: {}", page_data.len()));
    ui::print_info(&format!("Unique tags found: {}", all_tags.len()));

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
                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .apply_modifier(UTF8_ROUND_CORNERS)
                    .set_header(vec![Cell::new("Tag")
                        .add_attribute(Attribute::Bold)
                        .fg(Color::Cyan)]);

                for tag in sorted_tags {
                    table.add_row(vec![tag]);
                }
                table.to_string()
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
            if show_pages {
                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .apply_modifier(UTF8_ROUND_CORNERS)
                    .set_content_arrangement(ContentArrangement::Dynamic)
                    .set_header(vec![
                        Cell::new("Title")
                            .add_attribute(Attribute::Bold)
                            .fg(Color::Cyan),
                        Cell::new("Space")
                            .add_attribute(Attribute::Bold)
                            .fg(Color::Cyan),
                        Cell::new("Tags")
                            .add_attribute(Attribute::Bold)
                            .fg(Color::Cyan),
                    ]);

                for page in page_data {
                    let tags = if page.tags.is_empty() {
                        "-".to_string()
                    } else {
                        page.tags.join(", ")
                    };
                    table.add_row(vec![
                        Cell::new(&page.title),
                        Cell::new(&page.space),
                        Cell::new(tags),
                    ]);
                }
                table.to_string()
            } else {
                let mut all_tags: HashSet<String> = HashSet::new();
                for page in page_data {
                    all_tags.extend(page.tags.iter().cloned());
                }

                if all_tags.is_empty() {
                    "No tags found.".to_string()
                } else {
                    let mut table = Table::new();
                    table
                        .load_preset(UTF8_FULL)
                        .apply_modifier(UTF8_ROUND_CORNERS)
                        .set_header(vec![Cell::new("Tag")
                            .add_attribute(Attribute::Bold)
                            .fg(Color::Cyan)]);

                    let mut sorted: Vec<_> = all_tags.into_iter().collect();
                    sorted.sort();
                    for tag in sorted {
                        table.add_row(vec![tag]);
                    }
                    table.to_string()
                }
            }
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
}
