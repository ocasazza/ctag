use crate::api::{filter_excluded_pages, sanitize_text, ConfluenceClient};
use crate::models::OutputFormat;
use crate::ui;
use anyhow::Result;
use clap::Args;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, ContentArrangement, Table};
use serde::Serialize;
use std::collections::HashSet;
use terminal_size::{terminal_size, Width};

#[derive(Args)]
pub struct GetArgs {
    /// CQL expression to match pages
    pub cql_expression: String,

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

#[derive(Serialize)]
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
    format: OutputFormat,
) -> Result<()> {
    let verbose = format == OutputFormat::Verbose;
    let is_structured = format == OutputFormat::Json || format == OutputFormat::Csv;

    if verbose {
        ui::print_header("GET TAGS");
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
    if let Some(s) = &spinner {
        s.finish_and_clear();
    }

    if pages.is_empty() {
        match format {
            OutputFormat::Json => println!("[]"),
            OutputFormat::Csv => println!(), // Empty CSV
            _ => ui::print_warning("No pages found matching the CQL expression."),
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
        if let Some(s) = &spinner {
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

    // Collect page data with tags
    if verbose {
        ui::print_step("Retrieving tags for pages...");
    }
    let mut page_data = Vec::new();
    let mut all_tags = HashSet::new();

    let progress = if show_progress && !is_structured {
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
            .content
            .as_ref()
            .and_then(|c| c.space.as_ref())
            .and_then(|s| s.name.as_deref())
            .or_else(|| page.space.as_ref().and_then(|s| s.name.as_deref())) // Added check for page.space
            .or_else(|| {
                page.result_global_container
                    .as_ref()
                    .and_then(|c| c.title.as_deref())
            })
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

        if let Some(p) = &progress {
            p.inc(1);
        }
    }

    if let Some(p) = &progress {
        p.finish_and_clear();
    }

    // Generate output
    let output_content = if args.tags_only {
        format_tags_only(&all_tags, &format)
    } else {
        format_page_data(&page_data, &format, args.show_pages, client.base_url())
    };

    // Output results
    if let Some(file_path) = args.output_file {
        std::fs::write(&file_path, output_content)?;
        if verbose {
            ui::print_success(&format!("Results saved to {}", file_path));
        }
    } else {
        println!("{}", output_content);
    }

    if verbose {
        eprintln!();
        ui::print_info(&format!("Total pages processed: {}", page_data.len()));
        ui::print_info(&format!("Unique tags found: {}", all_tags.len()));
    }

    Ok(())
}

fn format_tags_only(tags: &HashSet<String>, format: &OutputFormat) -> String {
    let mut sorted_tags: Vec<_> = tags.iter().collect();
    sorted_tags.sort();
    match format {
        OutputFormat::Json => serde_json::to_string_pretty(&sorted_tags).unwrap_or_default(),
        OutputFormat::Csv => {
            let mut wtr = csv::Writer::from_writer(vec![]);
            #[derive(Serialize)]
            struct TagCsv<'a> {
                tag: &'a str,
            }
            for tag in sorted_tags {
                wtr.serialize(TagCsv { tag }).unwrap();
            }
            String::from_utf8(wtr.into_inner().unwrap()).unwrap()
        }
        OutputFormat::Simple | OutputFormat::Verbose => {
            if sorted_tags.is_empty() {
                return "No tags found.".to_string();
            }
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

fn format_page_data(
    page_data: &[PageData],
    format: &OutputFormat,
    show_pages: bool,
    base_url: &str,
) -> String {
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
        OutputFormat::Csv => {
            let mut wtr = csv::Writer::from_writer(vec![]);
            if show_pages {
                #[derive(Serialize)]
                struct PageDataCsv<'a> {
                    id: &'a str,
                    title: &'a str,
                    space: &'a str,
                    tags: String,
                }

                for page in page_data {
                    wtr.serialize(PageDataCsv {
                        id: &page.id,
                        title: &page.title,
                        space: &page.space,
                        tags: page.tags.join(", "),
                    })
                    .unwrap();
                }
            } else {
                let mut all_tags: HashSet<String> = HashSet::new();
                for page in page_data {
                    all_tags.extend(page.tags.iter().cloned());
                }
                let mut sorted: Vec<_> = all_tags.into_iter().collect();
                sorted.sort();

                #[derive(Serialize)]
                struct TagCsv<'a> {
                    tag: &'a str,
                }

                for tag in sorted {
                    wtr.serialize(TagCsv { tag: &tag }).unwrap();
                }
            }
            String::from_utf8(wtr.into_inner().unwrap()).unwrap()
        }
        OutputFormat::Simple | OutputFormat::Verbose => {
            if page_data.is_empty() {
                return "No pages found.".to_string();
            }
            if show_pages {
                let mut table = Table::new();
                table
                    .load_preset(UTF8_FULL)
                    .apply_modifier(UTF8_ROUND_CORNERS)
                    .set_content_arrangement(ContentArrangement::Dynamic);

                let width = terminal_size().map(|(Width(w), _)| w).unwrap_or(120);
                table.set_width(width.saturating_sub(4));

                table.set_header(vec![
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

                use comfy_table::{ColumnConstraint::*, Width::*};
                table
                    .column_mut(0)
                    .unwrap()
                    .set_constraint(LowerBoundary(Fixed(40)));
                table
                    .column_mut(1)
                    .unwrap()
                    .set_constraint(LowerBoundary(Fixed(15)));
                table
                    .column_mut(2)
                    .unwrap()
                    .set_constraint(LowerBoundary(Fixed(20)));

                for page in page_data {
                    let tags = if page.tags.is_empty() {
                        "-".to_string()
                    } else {
                        page.tags.join(", ")
                    };

                    let title_content = if format == &OutputFormat::Verbose {
                        // Create a terminal hyperlink if in verbose mode
                        format!(
                            "\x1b]8;;{}/wiki/pages/viewpage.action?pageId={}\x1b\\{}\x1b]8;;\x1b\\",
                            base_url, page.id, page.title
                        )
                    } else {
                        page.title.clone()
                    };

                    table.add_row(vec![
                        Cell::new(title_content),
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
        let out = format_tags_only(&tags, &OutputFormat::Simple);
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
