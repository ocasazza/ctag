use crate::ui;
use anyhow::Result;
use clap::Args;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, Color, Table};
use ctag::api::{sanitize_text, ConfluenceClient};
use ctag::models::OutputFormat;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Args)]
#[command(after_help = "\
EXAMPLES:
  # Get all pages with their tags
  ctag get 'space = DOCS'

  # Show only unique tags across all pages
  ctag get 'space = DOCS' --tags-only

  # Output as JSON
  ctag get 'space = DOCS' --format json

  # Save results to a file
  ctag get 'space = DOCS' --output-file results.json

  # Get tags from recently modified pages
  ctag get 'space = DOCS AND lastmodified > -30d'

  # Get tags in CSV format
  ctag get 'label = migration' --format csv --output-file migration-tags.csv
")]
pub struct GetArgs {
    /// CQL expression to match pages
    pub cql_expression: String,

    /// Include page titles and spaces in output
    #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    ancestors: Vec<String>,
    url: String,
}

pub fn run(
    args: GetArgs,
    client: &ConfluenceClient,
    show_progress: bool,
    format: OutputFormat,
) -> Result<()> {
    let verbose = format.is_verbose();
    let is_structured = format.is_structured();
    if verbose {
        ui::print_header("GET TAGS");
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
    // Collect page data with tags - use parallel processing for large sets
    if verbose {
        ui::print_step("Retrieving tags for pages...");
    }
    let progress = if show_progress && !is_structured {
        Some(ui::create_progress_bar(pages.len() as u64))
    } else {
        None
    };

    // Use rayon for parallel tag fetching on large datasets
    use rayon::prelude::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    let progress_counter = AtomicUsize::new(0);
    let page_data: Vec<PageData> = pages
        .par_iter()
        .filter_map(|page| {
            let content = page.content.as_ref()?;
            let page_id = content.id.as_ref()?;
            let title = sanitize_text(page.title.as_deref().unwrap_or("Unknown"));
            let space = page.space_name().to_string();
            let tags = client.get_page_tags(page_id).unwrap_or_default();
            // Extract ancestor titles (they come in order from root to immediate parent)
            let ancestors: Vec<String> = content
                .ancestors
                .iter()
                .filter_map(|a| a.title.clone())
                .map(|t| sanitize_text(&t))
                .collect();

            let url = format!(
                "{}/wiki/pages/viewpage.action?pageId={}",
                client.base_url().trim_end_matches('/'),
                page_id
            );

            // Update progress
            let count = progress_counter.fetch_add(1, Ordering::Relaxed);
            if let Some(ref p) = progress {
                p.set_position((count + 1) as u64);
            }
            Some(PageData {
                id: page_id.clone(),
                title,
                space,
                tags,
                ancestors,
                url,
            })
        })
        .collect();

    let mut all_tags = HashSet::new();
    for pd in &page_data {
        all_tags.extend(pd.tags.iter().cloned());
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

/// Format page data as either a tree view (verbose) or path format (simple).
/// - Verbose: Shows hierarchical tree structure with ├── └── connectors
/// - Simple: Shows path format like /Space/Parent/Page [tag1, tag2]
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
                    path: String,
                    space: &'a str,
                    tags: String,
                    url: &'a str,
                }

                for page in page_data {
                    let path = build_page_path(&page.space, &page.ancestors, &page.title);
                    wtr.serialize(PageDataCsv {
                        id: &page.id,
                        path,
                        space: &page.space,
                        tags: page.tags.join(", "),
                        url: &page.url,
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
        OutputFormat::Simple => {
            if page_data.is_empty() {
                return "No pages found.".to_string();
            }
            if show_pages {
                format_as_paths(page_data, base_url)
            } else {
                format_tags_as_table(page_data)
            }
        }
        OutputFormat::Verbose => {
            if page_data.is_empty() {
                return "No pages found.".to_string();
            }
            if show_pages {
                format_as_tree(page_data, base_url)
            } else {
                format_tags_as_table(page_data)
            }
        }
    }
}

// Use shared functions from ui module
use crate::ui::{
    build_page_path, format_directory, format_space, format_tags_list, make_page_clickable,
};

/// Format pages as simple path format: /Space/Parent/Page [tag1, tag2]
fn format_as_paths(page_data: &[PageData], base_url: &str) -> String {
    let mut lines: Vec<String> = Vec::new();

    // Sort pages by their full path for consistent output
    let mut sorted_pages: Vec<_> = page_data.iter().collect();
    sorted_pages.sort_by(|a, b| {
        let path_a = build_page_path(&a.space, &a.ancestors, &a.title);
        let path_b = build_page_path(&b.space, &b.ancestors, &b.title);
        path_a.cmp(&path_b)
    });
    for page in sorted_pages {
        let path = build_page_path(&page.space, &page.ancestors, &page.title);
        let tags = format_tags_list(&page.tags);
        let clickable_path = make_page_clickable(&path, &page.id, base_url);
        lines.push(format!("{} {}", clickable_path, tags));
    }
    lines.join("\n")
}

/// Format pages as a tree structure similar to the `tree` command
fn format_as_tree(page_data: &[PageData], base_url: &str) -> String {
    use std::collections::BTreeMap;

    // Build a tree structure: Map<space, Map<path_component, children>>
    // We'll use a simple approach: collect all paths and render them as a tree

    #[derive(Default)]
    struct TreeNode {
        children: BTreeMap<String, TreeNode>,
        // If this node is a page (leaf), store page info
        page_info: Option<(String, String, Vec<String>)>, // (id, title, tags)
    }

    let mut root: BTreeMap<String, TreeNode> = BTreeMap::new();

    // Insert all pages into the tree
    for page in page_data {
        // Path components: space -> ancestors -> title
        let space_node = root.entry(page.space.clone()).or_default();

        let mut current = space_node;
        for ancestor in &page.ancestors {
            current = current.children.entry(ancestor.clone()).or_default();
        }

        // Insert the page itself
        let page_node = current.children.entry(page.title.clone()).or_default();
        page_node.page_info = Some((page.id.clone(), page.title.clone(), page.tags.clone()));
    }

    fn render_tree(
        node: &BTreeMap<String, TreeNode>,
        prefix: &str,
        base_url: &str,
        is_root: bool,
    ) -> Vec<String> {
        let mut lines = Vec::new();
        let entries: Vec<_> = node.iter().collect();
        let count = entries.len();

        for (i, (name, child)) in entries.iter().enumerate() {
            let is_last = i == count - 1;
            let connector = if is_root {
                ""
            } else if is_last {
                "└── "
            } else {
                "├── "
            };
            let child_prefix = if is_root {
                prefix.to_string()
            } else if is_last {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };

            // Format this node
            let display_name = if let Some((ref id, ref _title, ref tags)) = child.page_info {
                // This is a page - make it clickable and show tags
                let tags_str = format_tags_list(tags);
                let clickable = make_page_clickable(name, id, base_url);
                format!("{}{}{} {}", prefix, connector, clickable, tags_str)
            } else {
                // This is just a container (space or parent page not in results)
                format!("{}{}{}", prefix, connector, format_directory(name))
            };

            lines.push(display_name);

            // Recurse into children
            if !child.children.is_empty() {
                lines.extend(render_tree(&child.children, &child_prefix, base_url, false));
            }
        }

        lines
    }

    // Render each space as a root
    let mut all_lines = Vec::new();
    let spaces: Vec<_> = root.iter().collect();
    let space_count = spaces.len();

    for (i, (space_name, space_node)) in spaces.iter().enumerate() {
        // Space header with color
        all_lines.push(format_space(space_name));

        // Render children of this space
        let is_last_space = i == space_count - 1;
        let _ = is_last_space; // We don't need different prefix for last space
        all_lines.extend(render_tree(&space_node.children, "", base_url, false));

        // Add blank line between spaces (except after last)
        if i < space_count - 1 {
            all_lines.push(String::new());
        }
    }

    all_lines.join("\n")
}

/// Format tags only as a table (when show_pages is false)
fn format_tags_as_table(page_data: &[PageData]) -> String {
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

    #[test]
    fn build_page_path_basic() {
        let path = build_page_path("MySpace", &[], "MyPage");
        assert_eq!(path, "/MySpace/MyPage");
    }

    #[test]
    fn build_page_path_with_ancestors() {
        let ancestors = vec!["Parent".to_string(), "Child".to_string()];
        let path = build_page_path("DOCS", &ancestors, "GrandChild");
        assert_eq!(path, "/DOCS/Parent/Child/GrandChild");
    }

    #[test]
    fn format_tags_list_empty() {
        let tags: Vec<String> = vec![];
        let result = format_tags_list(&tags);
        assert!(result.contains("[]"), "Expected [] in output: {}", result);
    }

    #[test]
    fn format_tags_list_single() {
        let tags = vec!["migration".to_string()];
        let result = format_tags_list(&tags);
        assert!(
            result.contains("[migration]"),
            "Expected [migration] in output: {}",
            result
        );
    }

    #[test]
    fn format_tags_list_multiple() {
        let tags = vec!["alpha".to_string(), "beta".to_string()];
        let result = format_tags_list(&tags);
        assert!(
            result.contains("[alpha, beta]"),
            "Expected [alpha, beta] in output: {}",
            result
        );
    }

    #[test]
    fn format_as_paths_produces_sorted_output() {
        let pages = vec![
            PageData {
                id: "2".to_string(),
                title: "Zebra".to_string(),
                space: "DOCS".to_string(),
                tags: vec!["z-tag".to_string()],
                ancestors: vec![],
                url: "http://example.com/2".to_string(),
            },
            PageData {
                id: "1".to_string(),
                title: "Alpha".to_string(),
                space: "DOCS".to_string(),
                tags: vec!["a-tag".to_string()],
                ancestors: vec![],
                url: "http://example.com/1".to_string(),
            },
        ];
        let output = format_as_paths(&pages, "https://example.atlassian.net");
        let lines: Vec<&str> = output.lines().collect();
        // Should be sorted alphabetically by path
        assert!(lines[0].contains("Alpha"));
        assert!(lines[1].contains("Zebra"));
    }

    #[test]
    fn format_as_tree_single_page() {
        let pages = vec![PageData {
            id: "123".to_string(),
            title: "TestPage".to_string(),
            space: "MYSPACE".to_string(),
            tags: vec!["tag1".to_string()],
            ancestors: vec![],
            url: "http://example.com/123".to_string(),
        }];
        let output = format_as_tree(&pages, "https://example.atlassian.net");
        // Should contain the space name and page
        assert!(output.contains("MYSPACE"));
        assert!(output.contains("TestPage"));
        assert!(output.contains("[tag1]"));
    }

    #[test]
    fn format_as_tree_with_hierarchy() {
        let pages = vec![
            PageData {
                id: "1".to_string(),
                title: "ChildPage".to_string(),
                space: "DOCS".to_string(),
                tags: vec!["child-tag".to_string()],
                ancestors: vec!["ParentPage".to_string()],
                url: "http://example.com/1".to_string(),
            },
            PageData {
                id: "2".to_string(),
                title: "ParentPage".to_string(),
                space: "DOCS".to_string(),
                tags: vec!["parent-tag".to_string()],
                ancestors: vec![],
                url: "http://example.com/2".to_string(),
            },
        ];
        let output = format_as_tree(&pages, "https://example.atlassian.net");
        // Should show hierarchy with tree connectors
        assert!(output.contains("DOCS"));
        assert!(output.contains("ParentPage"));
        assert!(output.contains("ChildPage"));
        // The child should be indented under parent (has tree connector)
        assert!(output.contains("└──") || output.contains("├──"));
    }

    #[test]
    fn format_page_data_simple_with_ancestors() {
        let pages = vec![PageData {
            id: "123".to_string(),
            title: "DeepPage".to_string(),
            space: "MYSPACE".to_string(),
            tags: vec!["important".to_string()],
            ancestors: vec!["Level1".to_string(), "Level2".to_string()],
            url: "http://example.com/123".to_string(),
        }];
        let output = format_page_data(&pages, &OutputFormat::Simple, true, "https://example.com");
        // Simple mode should show path format
        assert!(output.contains("/MYSPACE/Level1/Level2/DeepPage"));
        assert!(output.contains("[important]"));
    }

    #[test]
    fn format_page_data_json_includes_ancestors() {
        let pages = vec![PageData {
            id: "123".to_string(),
            title: "TestPage".to_string(),
            space: "MYSPACE".to_string(),
            tags: vec!["tag1".to_string()],
            ancestors: vec!["Parent".to_string()],
            url: "http://example.com/123".to_string(),
        }];
        let output = format_page_data(&pages, &OutputFormat::Json, true, "https://example.com");
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&output).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0]["ancestors"][0], "Parent");
    }

    #[test]
    fn format_page_data_csv_includes_path() {
        let pages = vec![PageData {
            id: "123".to_string(),
            title: "TestPage".to_string(),
            space: "MYSPACE".to_string(),
            tags: vec!["tag1".to_string()],
            ancestors: vec!["Parent".to_string()],
            url: "http://example.com/123".to_string(),
        }];
        let output = format_page_data(&pages, &OutputFormat::Csv, true, "https://example.com");
        // CSV should have path column
        assert!(output.contains("/MYSPACE/Parent/TestPage"));
    }
}
