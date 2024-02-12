# wls

wls (web ls) makes it easy to crawl multiple sitemaps and list URLs. It can even automatically find sitemaps for a domain using robots.txt.

## Usage

wls accepts multiple domains/sitemaps as arguments, and will print all found URLs to stdout:

```sh
$ wls docs.rs > urls.txt

$ head -n 6 urls.txt 
https://docs.rs/A-1/latest/A_1/
https://docs.rs/A-1/latest/A_1/all.html
https://docs.rs/A5/latest/A5/
https://docs.rs/A5/latest/A5/all.html
https://docs.rs/AAAA/latest/AAAA/
https://docs.rs/AAAA/latest/AAAA/all.html

$ grep /all.html urls.txt | wc -l
113191
# that's a lot of crates!
```

If an argument does not contain a slash, it is treated as a domain, and wls will automatically attempt to find sitemaps using robots.txt. For example, [docs.rs](https://docs.rs/) uses the `Sitemap:` directive in [its robots.txt file](https://docs.rs/robots.txt), so the following commands are equivalent:

```sh
$ wls docs.rs
$ wls https://docs.rs/robots.txt
$ wls https://docs.rs/sitemap.xml
```

wls will print logs to stderr when `-v/--verbose` is enabled:

```sh
$ wls -v docs.rs
   Found 1 sitemaps
    in robotstxt with url: https://docs.rs/robots.txt

   Found 26 sitemaps
    in sitemap with url: https://docs.rs/sitemap.xml
    in robotstxt with url: https://docs.rs/robots.txt

   Found 15934 URLs
    in sitemap with url: https://docs.rs/-/sitemap/a/sitemap.xml
    in sitemap with url: https://docs.rs/sitemap.xml
    in robotstxt with url: https://docs.rs/robots.txt

   Found 11170 URLs
    in sitemap with url: https://docs.rs/-/sitemap/b/sitemap.xml
    in sitemap with url: https://docs.rs/sitemap.xml
    in robotstxt with url: https://docs.rs/robots.txt

  ...
```

More options are available too:

```
$ wls --help
Usage: wls [OPTIONS] <URLS>...

Arguments:
  <URLS>...  Domains/sitemaps to crawl

Options:
  -U, --user-agent <USER_AGENT>  Browser to identify as [default: wls/0.1.0]
  -T, --timeout <SECONDS>        Maximum response time [default: 30]
  -w, --wait <SECONDS>           Delay between requests [default: 0]
  -v, --verbose                  Enable logs
  -h, --help                     Print help
  -V, --version                  Print version
```
