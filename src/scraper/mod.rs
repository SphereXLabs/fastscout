use std::collections::HashSet;

use scraper::{Html, Node, Selector};
use serde::Serialize;

use crate::error::{ErrorContext, Result};

#[derive(Debug, Clone, Serialize)]
pub struct Page {
    pub url: String,
    pub title: Option<String>,
    pub text: String,
    pub links: Vec<String>,
}

pub async fn fetch_page(url: &str) -> Result<Page> {
    let target = reqwest::Url::parse(url).scrape_context(url)?;

    let client = reqwest::Client::builder()
        .user_agent("FastScout/0.1")
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .scrape_context(url)?;

    let response = client.get(target.clone()).send().await.scrape_context(url)?;
    let status = response.status();
    if !status.is_success() {
        anyhow::bail!("HTTP {status} — {url}");
    }

    let body = response.text().await.scrape_context(url)?;
    let document = Html::parse_document(&body);

    Ok(Page {
        url: target.to_string(),
        title: extract_title(&document),
        text: visible_text(&document),
        links: extract_links(&document, &target),
    })
}

fn extract_title(document: &Html) -> Option<String> {
    let selector = Selector::parse("title").expect("geçerli seçici");
    document
        .select(&selector)
        .next()
        .map(|node| collapse_whitespace(&node.text().collect::<String>()))
        .filter(|title| !title.is_empty())
}

fn visible_text(document: &Html) -> String {
    let mut parts = Vec::new();
    for node in document.tree.root().descendants() {
        let Some(text) = node.value().as_text() else {
            continue;
        };
        let hidden = node.ancestors().any(|ancestor| {
            matches!(ancestor.value(), Node::Element(element) if is_hidden(element.name()))
        });
        if hidden {
            continue;
        }
        let cleaned = collapse_whitespace(text);
        if !cleaned.is_empty() {
            parts.push(cleaned);
        }
    }
    parts.join("\n")
}

fn is_hidden(name: &str) -> bool {
    matches!(
        name,
        "script" | "style" | "noscript" | "template" | "svg"
    )
}

fn extract_links(document: &Html, base: &reqwest::Url) -> Vec<String> {
    let selector = Selector::parse("a[href]").expect("geçerli seçici");
    let mut seen = HashSet::new();
    let mut links = Vec::new();

    for anchor in document.select(&selector) {
        let Some(href) = anchor.value().attr("href") else {
            continue;
        };
        let href = href.trim();
        if href.is_empty()
            || href.starts_with('#')
            || href.starts_with("javascript:")
            || href.starts_with("mailto:")
        {
            continue;
        }
        let Ok(absolute) = base.join(href) else {
            continue;
        };
        let value = absolute.to_string();
        if seen.insert(value.clone()) {
            links.push(value);
        }
    }
    links
}

fn collapse_whitespace(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ")
}
