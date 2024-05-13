#[macro_use]
extern crate tracing;

use std::{io, time::Duration};

use clap::Parser;
use color_eyre::eyre::{Context, Result};
use reqwest::blocking::ClientBuilder;
use tracing::level_filters::LevelFilter;
use tracing_error::ErrorLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};
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

    /// Enable cookies while crawling
    #[arg(short, long)]
    cookies: bool,
    /// Disable certificate verification
    #[arg(short = 'k', long)]
    insecure: bool,
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

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .pretty()
                .without_time()
                .with_file(false)
                .with_line_number(false)
                .with_level(false)
                .with_target(false)
                .with_writer(io::stderr)
                .with_filter(match cli.verbose {
                    true => LevelFilter::INFO,
                    false => LevelFilter::WARN,
                }),
        )
        .with(ErrorLayer::default())
        .init();
    color_eyre::install()?;

    let mut parsed = Vec::new();
    for url in cli.urls {
        let url = if url.contains("/") {
            url
        } else {
            // domain shorthand
            format!("https://{url}/robots.txt")
        };
        parsed.push(Url::parse(&url).wrap_err_with(|| format!("Failed to parse URL: {url}"))?);
    }
    debug!("{:#?}", parsed);

    let client = ClientBuilder::new()
        .cookie_store(cli.cookies)
        .danger_accept_invalid_certs(cli.insecure)
        .user_agent(cli.user_agent)
        .timeout(Duration::from_secs(cli.timeout))
        .build()?;
    let mut crawler = SitemapCrawler::new(client, Duration::from_secs(cli.wait));
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
