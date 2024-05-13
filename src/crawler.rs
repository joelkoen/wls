use std::{collections::HashSet, thread::sleep, time::Duration};

use color_eyre::eyre::{bail, Result};
use reqwest::blocking::Client;
use url::Url;

use crate::{robots::parse_robots, sitemap::parse_sitemap};

pub(crate) struct SitemapCrawler {
    client: Client,
    urls: HashSet<Url>,
    visited: HashSet<Url>,
    wait: Duration,
}

impl SitemapCrawler {
    pub(crate) fn new(client: Client, wait: Duration) -> Self {
        Self {
            client,
            urls: HashSet::new(),
            visited: HashSet::new(),
            wait,
        }
    }

    fn visit(&mut self, url: &Url) -> Result<Option<String>> {
        if self.visited.contains(&url) {
            warn!("Already visited");
            Ok(None)
        } else {
            self.visited.insert(url.clone());
            let body = self
                .client
                .get(url.as_str())
                .send()?
                .error_for_status()?
                .text()?;
            sleep(self.wait);
            Ok(Some(body))
        }
    }

    #[instrument(skip_all, fields(%url))]
    pub(crate) fn robotstxt(&mut self, url: Url) -> Result<()> {
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
    pub(crate) fn sitemap(&mut self, url: Url) -> Result<()> {
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

    pub(crate) fn urls(self) -> Vec<Url> {
        let mut x: Vec<_> = self.urls.into_iter().collect();
        x.sort();
        x
    }
}
