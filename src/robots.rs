use robotstxt::{parser::RobotsTxtParser, RobotsParseHandler};
use url::Url;

pub fn parse_robots(body: &str) -> Vec<Url> {
    let mut handler = RobotsSitemapHandler::new();
    let mut parser = RobotsTxtParser::new(body, &mut handler);
    parser.parse();

    handler.sitemaps
}

struct RobotsSitemapHandler {
    sitemaps: Vec<Url>,
}

impl RobotsSitemapHandler {
    fn new() -> Self {
        Self {
            sitemaps: Vec::new(),
        }
    }
}

impl RobotsParseHandler for RobotsSitemapHandler {
    fn handle_sitemap(&mut self, _: u32, value: &str) {
        if let Some(url) = Url::parse(value).ok() {
            self.sitemaps.push(url)
        } else {
            error!("Failed to parse URL: {value}")
        }
    }

    fn handle_robots_start(&mut self) {}
    fn handle_robots_end(&mut self) {}
    fn handle_user_agent(&mut self, _: u32, _: &str) {}
    fn handle_allow(&mut self, _: u32, _: &str) {}
    fn handle_disallow(&mut self, _: u32, _: &str) {}
    fn handle_unknown_action(&mut self, _: u32, _: &str, _: &str) {}
}
