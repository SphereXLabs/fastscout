//! FastScout hata katmanı. Tüm modüller `crate::error::Result` döner.

use anyhow::Context;

pub type Result<T> = anyhow::Result<T>;

pub trait ErrorContext<T> {
    fn scrape_context(self, url: &str) -> Result<T>;
    fn db_context(self, action: &str) -> Result<T>;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn scrape_context(self, url: &str) -> Result<T> {
        self.with_context(|| format!("sayfa alınamadı: {url}"))
    }

    fn db_context(self, action: &str) -> Result<T> {
        self.with_context(|| format!("veritabanı hatası ({action})"))
    }
}
