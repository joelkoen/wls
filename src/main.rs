#[macro_use]
extern crate log;

use anyhow::{Context, Result};
use clap::Parser;
use env_logger::Env;
use url::Url;

use crate::crawler::SitemapCrawler;

mod crawler;
mod robots;
mod sitemap;

#[derive(Debug, Parser)]
struct Cli {
    #[arg(required = true)]
    urls: Vec<String>,

    #[arg(short, long, group = "log")]
    quiet: bool,
    #[arg(short, long, group = "log")]
    verbose: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    env_logger::init_from_env(Env::new().default_filter_or(if cli.verbose {
        "debug"
    } else if cli.quiet {
        "error"
    } else {
        "info"
    }));
    debug!("{:#?}", cli);

    let mut parsed = Vec::new();
    for url in cli.urls {
        let url = if url.contains("/") {
            url
        } else {
            // domain shorthand
            format!("https://{url}/robots.txt")
        };
        parsed.push(Url::parse(&url).with_context(|| format!("Failed to parse URL: {url}"))?);
    }
    debug!("{:#?}", parsed);

    let mut crawler = SitemapCrawler::new();
    for url in parsed {
        info!("Crawling {url}");
        if url.path() == "/robots.txt" {
            crawler.crawl_robots(url)?;
        } else {
            crawler.crawl_sitemap(url)?;
        }
    }

    let urls = crawler.urls();
    info!("Found {} URLs", urls.len());
    let urls: Vec<_> = urls.iter().map(|x| x.as_str()).collect();
    println!("{}", urls.join("\n"));

    Ok(())
}
