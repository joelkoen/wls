use std::collections::HashSet;

use anyhow::Result;
use reqwest::blocking::{Client, ClientBuilder};
use url::Url;

use crate::{robots::parse_robots, sitemap::parse_sitemap};

static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub struct SitemapCrawler {
    client: Client,
    urls: HashSet<Url>,
    visited: HashSet<Url>,
}

impl SitemapCrawler {
    pub fn new() -> Self {
        Self {
            client: ClientBuilder::new().user_agent(USER_AGENT).build().unwrap(),
            urls: HashSet::new(),
            visited: HashSet::new(),
        }
    }

    fn get(&mut self, url: Url) -> Result<String> {
        Ok(self.client.get(url).send()?.error_for_status()?.text()?)
    }

    pub fn crawl_robots(&mut self, url: Url) -> Result<()> {
        if self.visited.contains(&url) {
            warn!("{url} has already been visited");
            return Ok(());
        }

        let sitemaps = parse_robots(&self.get(url.clone())?);
        for sitemap in sitemaps {
            info!("{sitemap} (from {url})");
            self.crawl_sitemap(sitemap)?;
        }

        Ok(())
    }

    pub fn crawl_sitemap(&mut self, url: Url) -> Result<()> {
        if self.visited.contains(&url) {
            warn!("{url} has already been visited");
            return Ok(());
        }

        let (urls, sitemaps) = parse_sitemap(&self.get(url.clone())?)?;
        self.visited.insert(url.clone());

        if urls.len() > 0 {
            info!("Found {} URLs", urls.len());
            self.urls.extend(urls);
        }

        for sitemap in sitemaps {
            info!("{sitemap} (from {url})");
            self.crawl_sitemap(sitemap)?;
        }

        Ok(())
    }

    pub fn urls(self) -> Vec<Url> {
        let mut x: Vec<_> = self.urls.into_iter().collect();
        x.sort();
        x
    }
}
