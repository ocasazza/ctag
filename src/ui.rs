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

pub fn print_summary(
    total: usize,
    processed: usize,
    skipped: usize,
    success: usize,
    failed: usize,
) {
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
        Cell::new(total.to_string()).fg(Color::White),
    ]);
    table.add_row(vec![
        Cell::new("Processed").fg(Color::Blue),
        Cell::new(processed.to_string()).fg(Color::Blue),
    ]);
    table.add_row(vec![
        Cell::new("Skipped").fg(Color::Yellow),
        Cell::new(skipped.to_string()).fg(Color::Yellow),
    ]);
    table.add_row(vec![
        Cell::new("Successful").fg(Color::Green),
        Cell::new(success.to_string()).fg(Color::Green),
    ]);
    table.add_row(vec![
        Cell::new("Failed").fg(Color::Red),
        Cell::new(failed.to_string()).fg(Color::Red),
    ]);
    eprintln!("\n{}", "Execution Summary".bold().bright_white());
    println!("{table}");
}

pub fn print_header(title: &str) {
    println!("\n{}", "=".repeat(title.len() + 4).dimmed());
    println!("  {}", title.bold().bright_white());
    println!("{}\n", "=".repeat(title.len() + 4).dimmed());
}

pub fn print_page_action(action: &str, title: &str, space: &str) {
    println!(
        "{} {} {} {} {}",
        "→".bright_blue().bold(),
        action.bold(),
        "\"".dimmed(),
        title.bright_white(),
        "\"".dimmed()
    );
    println!("  {} {}", "in space".dimmed(), space.cyan());
}
