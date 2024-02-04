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

#[derive(Debug, Parser)]
struct Cli {
    #[arg(required = true)]
    urls: Vec<String>,

    #[arg(short = 'T', long, default_value_t = 30)]
    timeout: u64,
    #[arg(short, long, default_value_t = 0)]
    wait: u64,

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
