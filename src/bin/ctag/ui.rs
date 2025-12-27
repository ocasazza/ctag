use colored::*;
use indicatif::{ProgressBar, ProgressStyle};

pub fn print_step(msg: &str) {
    eprintln!("{} {}", "•".bold().blue(), msg.bold());
}

pub fn print_substep(msg: &str) {
    eprintln!("  {} {}", "-".dimmed(), msg);
}

pub fn print_success(msg: &str) {
    eprintln!("{} {}", "✓".bold().green(), msg.green());
}

pub fn print_error(msg: &str) {
    eprintln!("{} {}", "✗".bold().red(), msg.red());
}

pub fn print_warning(msg: &str) {
    eprintln!("{} {}", "!".bold().yellow(), msg.yellow());
}

pub fn print_info(msg: &str) {
    eprintln!("{} {}", "i".bold().blue(), msg.blue());
}

pub fn print_dry_run(msg: &str) {
    eprintln!("{} {}", "[DRY RUN]".bold().purple(), msg.dimmed());
}

pub fn create_progress_bar(len: u64) -> ProgressBar {
    let pb = ProgressBar::new(len);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
            .unwrap()
            .progress_chars("━╸ "),
    );
    pb
}

pub fn create_pagination_spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ")
            .template("{spinner:.green} {msg} ({pos} pages)")
            .unwrap(),
    );
    pb.set_message(msg.to_string());
    pb
}

// Formatters for results
pub fn print_summary(results: &ctag::models::ProcessResults, format: ctag::models::OutputFormat) {
    match format {
        ctag::models::OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(results).unwrap());
        }
        ctag::models::OutputFormat::Csv => {
            #[derive(serde::Serialize)]
            struct CsvSummary {
                total: usize,
                processed: usize,
                skipped: usize,
                success: usize,
                failed: usize,
                aborted: bool,
                tags_added: usize,
                tags_removed: usize,
            }
            let summary = CsvSummary {
                total: results.total,
                processed: results.processed,
                skipped: results.skipped,
                success: results.success,
                failed: results.failed,
                aborted: results.aborted,
                tags_added: results.tags_added,
                tags_removed: results.tags_removed,
            };
            let mut wtr = csv::Writer::from_writer(std::io::stdout());
            wtr.serialize(summary).unwrap();
            wtr.flush().unwrap();
        }
        ctag::models::OutputFormat::Verbose => {
            print_summary_table(results);
        }
        ctag::models::OutputFormat::Simple => {
            print_summary_minimal(results);
        }
    }
}

fn print_summary_table(results: &ctag::models::ProcessResults) {
    use comfy_table::modifiers::UTF8_ROUND_CORNERS;
    use comfy_table::presets::UTF8_FULL;
    use comfy_table::*;
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec![
            Cell::new("Category")
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
            Cell::new("Count")
                .add_attribute(Attribute::Bold)
                .fg(Color::Cyan),
        ]);

    table.add_row(vec![
        Cell::new("Total Pages Found").add_attribute(Attribute::Bold),
        Cell::new(results.total.to_string()).fg(Color::White),
    ]);
    table.add_row(vec![
        Cell::new("Processed").fg(Color::Blue),
        Cell::new(results.processed.to_string()).fg(Color::Blue),
    ]);
    table.add_row(vec![
        Cell::new("Skipped").fg(Color::Yellow),
        Cell::new(results.skipped.to_string()).fg(Color::Yellow),
    ]);
    table.add_row(vec![
        Cell::new("Successful").fg(Color::Green),
        Cell::new(results.success.to_string()).fg(Color::Green),
    ]);
    table.add_row(vec![
        Cell::new("Failed").fg(Color::Red),
        Cell::new(results.failed.to_string()).fg(Color::Red),
    ]);
    if results.tags_added > 0 || results.tags_removed > 0 {
        table.add_row(vec![
            Cell::new("Tags Added").fg(Color::Green),
            Cell::new(results.tags_added.to_string()).fg(Color::Green),
        ]);
        table.add_row(vec![
            Cell::new("Tags Removed").fg(Color::Red),
            Cell::new(results.tags_removed.to_string()).fg(Color::Red),
        ]);
    }
    eprintln!("\n{}", "Execution Summary".bold().bright_white());
    println!("{table}");
}

pub fn print_summary_minimal(results: &ctag::models::ProcessResults) {
    let mut parts = Vec::new();

    parts.push(format!(
        "{} {}",
        "Processed:".bold(),
        results.processed.to_string().cyan()
    ));

    if results.success > 0 {
        parts.push(format!(
            "{} {}",
            "Success:".bold(),
            results.success.to_string().green()
        ));
    }

    if results.failed > 0 {
        parts.push(format!(
            "{} {}",
            "Failed:".bold(),
            results.failed.to_string().red()
        ));
    }

    if results.skipped > 0 {
        parts.push(format!(
            "{} {}",
            "Skipped:".bold(),
            results.skipped.to_string().yellow()
        ));
    }

    if results.tags_added > 0 {
        parts.push(format!(
            "{} {}",
            "Tags Added:".bold(),
            results.tags_added.to_string().green()
        ));
    }

    if results.tags_removed > 0 {
        parts.push(format!(
            "{} {}",
            "Tags Removed:".bold(),
            results.tags_removed.to_string().red()
        ));
    }

    println!("\n{}", parts.join(" | "));
}

pub fn print_header(title: &str) {
    eprintln!("\n{}", "=".repeat(title.len() + 4).dimmed());
    eprintln!("  {}", title.bold().bright_white());
    eprintln!("{}\n", "=".repeat(title.len() + 4).dimmed());
}

pub fn print_page_action(action: &str, title: &str, space: &str) {
    eprintln!(
        "{} {} {}",
        "→".bright_blue().bold(),
        action.bold(),
        title.bright_white()
    );
    eprintln!("  {} {}", "space:".dimmed(), space.cyan());
}

// ========== Shared Formatting Functions ==========
/// Format tags as a colorized bracketed list
/// - Green if has tags: [tag1, tag2]
/// - Dim if empty: []
pub fn format_tags_list(tags: &[String]) -> String {
    if tags.is_empty() {
        "\x1b[2m[]\x1b[0m".to_string()
    } else {
        format!("\x1b[32m[{}]\x1b[0m", tags.join(", "))
    }
}

/// Create a clickable hyperlink for terminal emulators that support OSC 8
pub fn make_clickable(text: &str, url: &str) -> String {
    format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", url, text)
}

/// Create a clickable page link using page ID
pub fn make_page_clickable(text: &str, page_id: &str, base_url: &str) -> String {
    let url = format!(
        "{}/wiki/pages/viewpage.action?pageId={}",
        base_url.trim_end_matches('/'),
        page_id
    );
    make_clickable(text, &url)
}

/// Build a page path like /Space/Parent/Child/PageTitle
pub fn build_page_path(space: &str, ancestors: &[String], title: &str) -> String {
    let mut parts = vec![space.to_string()];
    parts.extend(ancestors.iter().cloned());
    parts.push(title.to_string());
    format!("/{}", parts.join("/"))
}

/// Format a space name with color (bold cyan)
pub fn format_space(space: &str) -> String {
    format!("\x1b[1;36m{}\x1b[0m", space)
}

/// Format a directory/parent node name (bold blue)
pub fn format_directory(name: &str) -> String {
    format!("\x1b[1;34m{}\x1b[0m", name)
}
