use std::collections::HashSet;

use anyhow::{bail, Result};
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

    fn visit(&mut self, url: &Url) -> Result<Option<String>> {
        if self.visited.contains(&url) {
            warn!("Already visited");
            Ok(None)
        } else {
            self.visited.insert(url.clone());
            Ok(Some(
                self.client
                    .get(url.clone())
                    .send()?
                    .error_for_status()?
                    .text()?,
            ))
        }
    }

    #[instrument(skip_all, fields(%url))]
    pub fn robotstxt(&mut self, url: Url) -> Result<()> {
        if let Some(body) = self.visit(&url)? {
            let sitemaps = parse_robots(&body);
            if sitemaps.len() > 0 {
                info!("Found {} sitemaps", sitemaps.len());
                for sitemap in sitemaps {
                    self.sitemap(sitemap)?;
                }
            } else {
                bail!("Found 0 sitemaps");
            }
        }

        Ok(())
    }

    #[instrument(skip_all, fields(%url))]
    pub fn sitemap(&mut self, url: Url) -> Result<()> {
        if let Some(body) = self.visit(&url)? {
            let (urls, sitemaps) = parse_sitemap(&body)?;

            if urls.len() > 0 && sitemaps.len() > 0 {
                info!("Found {} URLs and {} sitemaps", urls.len(), sitemaps.len());
            } else if urls.len() > 0 {
                info!("Found {} URLs", urls.len());
            } else if sitemaps.len() > 0 {
                info!("Found {} sitemaps", sitemaps.len());
            } else {
                warn!("Nothing found");
            }

            self.urls.extend(urls);
            for sitemap in sitemaps {
                self.sitemap(sitemap)?;
            }
        }

        Ok(())
    }

    pub fn urls(self) -> Vec<Url> {
        let mut x: Vec<_> = self.urls.into_iter().collect();
        x.sort();
        x
    }
}
