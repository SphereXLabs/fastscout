mod db;
mod error;
mod scraper;

use std::path::PathBuf;

use clap::{Parser, ValueEnum};
use colored::Colorize;

use crate::error::Result;
use crate::scraper::Page;

#[derive(Parser, Debug)]
#[command(name = "fastscout", version, about = "Hızlı web sayfası keşif aracı")]
struct Cli {
    /// Taranacak sayfanın adresi
    #[arg(long)]
    url: String,

    /// Çıktı biçimi
    #[arg(long, value_enum, default_value = "text")]
    format: OutputFormat,

    /// SQLite dosyası
    #[arg(long, default_value = "fastscout.db")]
    db: PathBuf,
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
    let page = scraper::fetch_page(&cli.url).await?;
    let pool = db::connect(&cli.db.to_string_lossy()).await?;
    db::save_page(&pool, &page.url, page.title.as_deref(), &page.text).await?;

    match cli.format {
        OutputFormat::Text => print_text(&page),
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&page)?),
    }
    Ok(())
}

fn print_text(page: &Page) {
    println!("{} {}", "URL:".cyan().bold(), page.url);
    match &page.title {
        Some(title) => println!("{} {}", "Başlık:".cyan().bold(), title.green().bold()),
        None => println!("{}", "Başlık bulunamadı".yellow()),
    }
    println!(
        "{} {}",
        "Bağlantı:".cyan().bold(),
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
