use std::{collections::HashSet, thread::sleep, time::Duration};

use anyhow::{bail, Result};
use ureq::{Agent, AgentBuilder};
use url::Url;

use crate::{robots::parse_robots, sitemap::parse_sitemap};

pub(crate) struct SitemapCrawler {
    agent: Agent,
    urls: HashSet<Url>,
    visited: HashSet<Url>,
    wait: Duration,
}

impl SitemapCrawler {
    pub(crate) fn new(user_agent: &str, timeout: Duration, wait: Duration) -> Self {
        Self {
            agent: AgentBuilder::new()
                .user_agent(user_agent)
                .timeout(timeout)
                .build(),
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
            let body = self.agent.get(url.as_str()).call()?.into_string()?;
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
