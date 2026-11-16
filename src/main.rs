mod db;
mod error;
mod scraper;
mod urls;

use std::path::{Path, PathBuf};

use anyhow::Context;
use clap::{Parser, ValueEnum};
use colored::Colorize;

use crate::error::Result;
use crate::scraper::Page;

#[derive(Parser, Debug)]
#[command(
    name = "fastscout",
    version,
    about = "Asynchronous web scraper and page archive",
    arg_required_else_help = true
)]
struct Cli {
    /// Comma-separated page URLs. Repeat the flag to add more.
    #[arg(long, value_delimiter = ',', value_name = "URL")]
    url: Vec<String>,

    /// Text file with one URL per line. Blank lines and # comments are ignored.
    #[arg(long, value_name = "PATH")]
    file: Option<PathBuf>,

    /// Stdout format: a colorized report or pretty JSON.
    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,

    /// SQLite database file.
    #[arg(long, default_value = "fastscout.db")]
    db: PathBuf,

    /// Maximum number of pages fetched at the same time.
    #[arg(long, default_value_t = 10, value_parser = clap::value_parser!(u32).range(1..))]
    concurrency: u32,

    /// Write the scraped pages to this JSON file.
    #[arg(long, value_name = "PATH")]
    output: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(cli).await {
        eprintln!("{}", format!("{err:#}").red());
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    let file_contents = match &cli.file {
        Some(path) => Some(read_url_file(path)?),
        None => None,
    };
    let targets = urls::gather_urls(&cli.url, file_contents.as_deref())?;
    let pages = scraper::fetch_pages(&targets, cli.concurrency as usize).await?;

    let pool = db::connect(&cli.db.to_string_lossy()).await?;
    for page in &pages {
        db::save_page(&pool, &page.url, page.title.as_deref(), &page.text).await?;
    }

    match cli.format {
        OutputFormat::Text => print_pages(&pages),
        OutputFormat::Json => println!("{}", pages_json(&pages)?),
    }

    if let Some(path) = &cli.output {
        write_pages_json(path, &pages)?;
    }
    Ok(())
}

fn read_url_file(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .with_context(|| format!("failed to read URL file: {}", path.display()))
}

fn pages_json(pages: &[Page]) -> Result<String> {
    Ok(serde_json::to_string_pretty(pages)?)
}

fn write_pages_json(path: &Path, pages: &[Page]) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!("failed to create output directory: {}", parent.display())
            })?;
        }
    }
    std::fs::write(path, pages_json(pages)?)
        .with_context(|| format!("failed to write output file: {}", path.display()))?;
    Ok(())
}

fn print_pages(pages: &[Page]) {
    for (index, page) in pages.iter().enumerate() {
        if index > 0 {
            println!();
        }
        print_text(page);
    }
}

fn print_text(page: &Page) {
    println!("{} {}", "URL:".cyan().bold(), page.url);
    match &page.title {
        Some(title) => println!("{} {}", "Title:".cyan().bold(), title.green().bold()),
        None => println!("{}", "Title not found".yellow()),
    }
    println!(
        "{} {}",
        "Links:".cyan().bold(),
        page.links.len().to_string().yellow()
    );
    if !page.links.is_empty() {
        for link in &page.links {
            println!("  {}", link.yellow());
        }
    }
    println!("{}", "─".repeat(40).bright_black());
    println!("{}", page.text);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_page() -> Page {
        Page {
            url: "https://example.com/".to_string(),
            title: Some("Example".to_string()),
            text: "Hello".to_string(),
            links: vec!["https://example.com/a".to_string()],
        }
    }

    #[test]
    fn pages_json_is_an_array() {
        let json = pages_json(&[sample_page()]).unwrap();
        let parsed: Vec<Page> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, vec![sample_page()]);
    }

    #[test]
    fn write_pages_json_creates_parent_directory_and_file() {
        let dir = std::env::temp_dir().join(format!("fastscout-{}", std::process::id()));
        let path = dir.join("nested").join("pages.json");
        let _ = std::fs::remove_dir_all(&dir);

        write_pages_json(&path, &[sample_page()]).unwrap();

        let written = std::fs::read_to_string(&path).unwrap();
        assert!(written.starts_with('['));
        assert!(written.contains("https://example.com/"));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
