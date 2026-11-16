use std::collections::HashSet;
use std::sync::LazyLock;
use std::time::Duration;

use anyhow::Context;
use futures_util::stream::{self, StreamExt};
use scraper::{Html, Node, Selector};
use serde::{Deserialize, Serialize};

use crate::error::{ErrorContext, Result};

const USER_AGENT: &str = concat!("FastScout/", env!("CARGO_PKG_VERSION"));

static TITLE_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("title").expect("valid CSS selector"));

static LINK_SELECTOR: LazyLock<Selector> =
    LazyLock::new(|| Selector::parse("a[href]").expect("valid CSS selector"));

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Page {
    pub url: String,
    pub title: Option<String>,
    pub text: String,
    pub links: Vec<String>,
}

pub async fn fetch_pages(urls: &[String], concurrency: usize) -> Result<Vec<Page>> {
    let client = http_client()?;
    let limit = concurrency.max(1);
    let results = stream::iter(urls.iter().cloned())
        .map(|url| {
            let client = client.clone();
            async move { fetch_page_with(&client, &url).await }
        })
        .buffered(limit)
        .collect::<Vec<_>>()
        .await;

    results.into_iter().collect()
}

fn http_client() -> Result<reqwest::Client> {
    reqwest::Client::builder()
        .user_agent(USER_AGENT)
        .timeout(Duration::from_secs(30))
        .build()
        .context("failed to build HTTP client")
}

async fn fetch_page_with(client: &reqwest::Client, url: &str) -> Result<Page> {
    let target = reqwest::Url::parse(url).scrape_context(url)?;
    let response = client.get(target).send().await.scrape_context(url)?;
    let final_url = response.url().clone();
    let status = response.status();
    if !status.is_success() {
        anyhow::bail!("HTTP {status} — {final_url}");
    }

    let body = response.text().await.scrape_context(url)?;
    let document = Html::parse_document(&body);

    Ok(Page {
        url: final_url.to_string(),
        title: extract_title(&document),
        text: visible_text(&document),
        links: extract_links(&document, &final_url),
    })
}

fn extract_title(document: &Html) -> Option<String> {
    document
        .select(&TITLE_SELECTOR)
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
        "script" | "style" | "noscript" | "template" | "svg" | "head" | "title"
    )
}

fn extract_links(document: &Html, base: &reqwest::Url) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut links = Vec::new();

    for anchor in document.select(&LINK_SELECTOR) {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visible_text_omits_head_and_keeps_title_separate() {
        let document = Html::parse_document(
            "<html><head><title>Başlık</title><meta name=\"d\" content=\"gizli\"><style>body{}</style></head><body><p>Görünen</p><script>alert(1)</script></body></html>",
        );
        assert_eq!(visible_text(&document), "Görünen");
        assert_eq!(extract_title(&document).as_deref(), Some("Başlık"));
    }

    #[test]
    fn extract_links_resolves_against_the_given_base() {
        let document = Html::parse_document(
            r##"<a href="/a">A</a><a href="b">B</a><a href="#x">C</a><a href="mailto:a@b.c">D</a>"##,
        );
        let base = reqwest::Url::parse("https://www.example.com/dir/page").unwrap();
        assert_eq!(
            extract_links(&document, &base),
            vec![
                "https://www.example.com/a".to_string(),
                "https://www.example.com/dir/b".to_string(),
            ]
        );
    }
}
