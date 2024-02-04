use anyhow::{bail, Result};
use sitemap::reader::{SiteMapEntity, SiteMapReader};
use url::Url;

pub fn parse_sitemap(body: &str) -> Result<(Vec<Url>, Vec<Url>)> {
    let mut reader = SiteMapReader::new(body.as_bytes());

    let mut sitemaps = Vec::new();
    let mut urls = Vec::new();

    while let Some(entity) = reader.next() {
        match entity {
            SiteMapEntity::Url(entry) => {
                if let Some(url) = entry.loc.get_url() {
                    urls.push(url);
                }
            }
            SiteMapEntity::SiteMap(entry) => {
                if let Some(sitemap) = entry.loc.get_url() {
                    sitemaps.push(sitemap);
                }
            }
            SiteMapEntity::Err(err) => bail!("Failed to parse sitemap: {err}"),
        };
    }

    Ok((urls, sitemaps))
}
