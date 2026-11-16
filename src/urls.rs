use std::collections::HashSet;

use crate::error::Result;

/// Splits comma-separated values, trims whitespace, and keeps the first copy of each URL.
pub fn parse_url_list(values: impl IntoIterator<Item = impl AsRef<str>>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut urls = Vec::new();
    for value in values {
        for part in value.as_ref().split(',') {
            push_unique(&mut seen, &mut urls, part);
        }
    }
    urls
}

/// Reads one URL per line. Blank lines and `#` comments are ignored.
pub fn parse_url_file(contents: &str) -> Vec<String> {
    let lines = contents.lines().filter_map(|line| {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            None
        } else {
            Some(line.to_string())
        }
    });
    parse_url_list(lines)
}

pub fn gather_urls(cli_urls: &[String], file_contents: Option<&str>) -> Result<Vec<String>> {
    let mut urls = parse_url_list(cli_urls);
    if let Some(contents) = file_contents {
        let mut seen: HashSet<String> = urls.iter().cloned().collect();
        for url in parse_url_file(contents) {
            if seen.insert(url.clone()) {
                urls.push(url);
            }
        }
    }
    if urls.is_empty() {
        anyhow::bail!("provide at least one URL via --url or --file");
    }
    Ok(urls)
}

fn push_unique(seen: &mut HashSet<String>, urls: &mut Vec<String>, value: &str) {
    let value = value.trim();
    if value.is_empty() {
        return;
    }
    if seen.insert(value.to_string()) {
        urls.push(value.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_url_list_splits_trims_and_keeps_the_first_copy() {
        let urls = parse_url_list([
            " https://example.com/a , https://example.com/b ",
            "https://example.com/b",
            "https://example.com/c",
        ]);
        assert_eq!(
            urls,
            vec![
                "https://example.com/a".to_string(),
                "https://example.com/b".to_string(),
                "https://example.com/c".to_string(),
            ]
        );
    }

    #[test]
    fn parse_url_file_skips_blank_lines_and_comments() {
        let contents = "\n# comment\nhttps://example.com/a, https://example.com/b\n\n  https://example.com/a\n";
        assert_eq!(
            parse_url_file(contents),
            vec![
                "https://example.com/a".to_string(),
                "https://example.com/b".to_string(),
            ]
        );
    }

    #[test]
    fn gather_urls_merges_flags_and_file_without_duplicates() {
        let cli = vec!["https://example.com/a".to_string()];
        let file = "https://example.com/a\nhttps://example.com/b\n";
        assert_eq!(
            gather_urls(&cli, Some(file)).unwrap(),
            vec![
                "https://example.com/a".to_string(),
                "https://example.com/b".to_string(),
            ]
        );
    }

    #[test]
    fn gather_urls_requires_at_least_one_url() {
        let error = gather_urls(&[], Some("# nothing\n\n")).unwrap_err();
        assert!(error.to_string().contains("--url"));
    }
}
