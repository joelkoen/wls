#[macro_use]
extern crate tracing;

use std::{io, time::Duration};

use anyhow::{Context, Result};
use clap::Parser;
use tracing::level_filters::LevelFilter;
use url::Url;

use crate::crawler::SitemapCrawler;

mod crawler;
mod robots;
mod sitemap;

static USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

// NOTE: update README with changes to --help
#[derive(Debug, Parser)]
#[command(version)]
struct Cli {
    /// Domains/sitemaps to crawl
    #[arg(required = true)]
    urls: Vec<String>,

    /// Browser to identify as
    #[arg(short = 'U', long, default_value = USER_AGENT)]
    user_agent: String,

    /// Maximum response time
    #[arg(short = 'T', long, default_value_t = 30, value_name = "SECONDS")]
    timeout: u64,
    /// Delay between requests
    #[arg(short, long, default_value_t = 0, value_name = "SECONDS")]
    wait: u64,

    /// Enable logs
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    tracing_subscriber::fmt()
        .pretty()
        .without_time()
        .with_file(false)
        .with_line_number(false)
        .with_level(false)
        .with_target(false)
        .with_max_level(match cli.verbose {
            true => LevelFilter::INFO,
            false => LevelFilter::WARN,
        })
        .with_writer(io::stderr)
        .init();
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

    let mut crawler = SitemapCrawler::new(
        &cli.user_agent,
        Duration::from_secs(cli.timeout),
        Duration::from_secs(cli.wait),
    );
    for url in parsed {
        if url.path() == "/robots.txt" {
            crawler.robotstxt(url)?;
        } else {
            crawler.sitemap(url)?;
        }
    }

    let urls = crawler.urls();
    let urls: Vec<_> = urls.iter().map(|x| x.as_str()).collect();
    if urls.len() > 0 {
        info!("Found {} URLs", urls.len());
        println!("{}", urls.join("\n"));
    } else {
        error!("Found 0 URLs");
    }

    Ok(())
}
