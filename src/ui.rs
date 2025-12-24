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

pub fn create_spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏ ")
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message(msg.to_string());
    pb
}

// Formatters for results

pub fn print_summary(results: &crate::models::ProcessResults, format: crate::models::OutputFormat) {
    match format {
        crate::models::OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(results).unwrap());
        }
        crate::models::OutputFormat::Csv => {
            let mut wtr = csv::Writer::from_writer(std::io::stdout());
            wtr.serialize(results).unwrap();
            wtr.flush().unwrap();
        }
        crate::models::OutputFormat::Verbose => {
            print_summary_table(results);
        }
        crate::models::OutputFormat::Simple => {
            print_summary_minimal(results.processed, results.success, results.failed);
        }
    }
}

fn print_summary_table(results: &crate::models::ProcessResults) {
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
    eprintln!("\n{}", "Execution Summary".bold().bright_white());
    println!("{table}");
}

pub fn print_summary_minimal(processed: usize, success: usize, failed: usize) {
    println!(
        "\n{} {} | {} {} | {} {}",
        "Processed:".bold(),
        processed.to_string().cyan(),
        "Success:".bold(),
        success.to_string().green(),
        "Failed:".bold(),
        failed.to_string().red(),
    );
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
