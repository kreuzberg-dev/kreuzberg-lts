# URL Extraction

Fetch HTTP(S) URLs, extract their content, and optionally crawl from a seed URL — with a headless-browser fallback for JavaScript-rendered pages. URL ingestion reuses the same `extract` / `extract_batch` entry points as local files; a separate `map_url` function discovers the URL inventory of a site without extracting content.

!!! note "Feature gate"
    URL ingestion requires the `url-ingestion` Cargo feature (included in `full`). The headless-browser fallback additionally requires `url-ingestion-browser`. The pure-Rust `url-config-types` feature exposes the config types alone (no runtime). URL ingestion is not available on WASM.

## When to Use

- You extract content directly from web pages or remote documents (a PDF behind a URL).
- You crawl a site and extract every discovered page or downloaded document in one call.
- You need a site's URL inventory (sitemap / link graph) before committing to a full extraction run — use `map_url`.
- You scrape JavaScript-rendered pages that need a real browser to produce content.

## When Not to Use

- You only have local files or in-memory bytes. Use plain [`extract`](extraction.md) with a path or `bytes` input.
- You need a full-featured standalone crawler with its own pipeline. Use [crawlberg](https://github.com/xberg-io/crawlberg) directly.

## Entry Points

| Function        | Returns             | Purpose                                                          |
| --------------- | ------------------- | ---------------------------------------------------------------- |
| `extract`       | `ExtractionResult`  | Fetch an HTTP(S) URI and extract its content (single or crawl)   |
| `map_url`       | `MapResult`         | Discover reachable URLs without extracting document content      |

An `ExtractInput` with `kind = "uri"` and an `http`/`https` URI routes through the URL-ingestion path. `config.url` (a `UrlExtractionConfig`) controls fetch and crawl behaviour.

## Fetch and Extract a URL

`UrlExtractionConfig.mode` decides the shape of the run: `Auto` and `Document` fetch the single seed URL; `Crawl` follows links from the seed and extracts each discovered page or downloaded document.

=== "Python"

    ```python title="extract_url.py"
    from xberg import CrawlConfig, ExtractInput, ExtractionConfig, UrlExtractionConfig, UrlExtractionMode, extract

    config = ExtractionConfig(
        url=UrlExtractionConfig(
            mode=UrlExtractionMode.CRAWL,
            crawl=CrawlConfig(max_depth=1, max_pages=25),
        ),
    )
    output = await extract(ExtractInput(kind="uri", uri="https://example.com"), config=config)
    for result in output.results:
        print(result.content)
    ```

=== "TypeScript"

    ```typescript title="extract-url.ts"
    import { ExtractInputKind, UrlExtractionMode, extract } from "@xberg-io/xberg";

    const output = await extract(
      { kind: ExtractInputKind.Uri, uri: "https://example.com" },
      {
        url: {
          mode: UrlExtractionMode.Crawl,
          crawl: { maxDepth: 1, maxPages: 25 },
        },
      },
    );
    for (const result of output.results) {
      console.log(result.content);
    }
    ```

=== "Rust"

    ```rust title="extract_url.rs"
    use xberg::{extract, CrawlConfig, ExtractInput, ExtractionConfig, UrlExtractionConfig, UrlExtractionMode};

    let config = ExtractionConfig {
        url: UrlExtractionConfig {
            mode: UrlExtractionMode::Crawl,
            crawl: CrawlConfig {
                max_depth: Some(1),
                max_pages: Some(25),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };
    let output = extract(ExtractInput::from_uri("https://example.com"), &config).await?;
    for result in &output.results {
        println!("{}", result.content);
    }
    ```

`extract_batch` accepts a mix of URI and byte inputs, so remote pages and local files can be extracted in one call.

## Discover URLs with `map_url`

`map_url` builds a crawl engine from `config.crawl` and returns a `MapResult` — the set of discovered URLs — without extracting any document content. Use it to build a crawl queue or validate scope first.

=== "Python"

    ```python title="map_url.py"
    from xberg import CrawlConfig, UrlExtractionConfig, map_url

    result = await map_url(
        "https://example.com",
        UrlExtractionConfig(crawl=CrawlConfig(max_depth=2)),
    )
    for entry in result.urls:
        print(entry.url)
    ```

=== "TypeScript"

    ```typescript title="map-url.ts"
    import { mapUrl } from "@xberg-io/xberg";

    const result = await mapUrl("https://example.com", { crawl: { maxDepth: 2 } });
    for (const entry of result.urls ?? []) {
      console.log(entry.url);
    }
    ```

=== "Rust"

    ```rust title="map_url.rs"
    use xberg::{map_url, CrawlConfig, UrlExtractionConfig};

    let config = UrlExtractionConfig {
        crawl: CrawlConfig {
            max_depth: Some(2),
            ..Default::default()
        },
        ..Default::default()
    };
    let result = map_url("https://example.com", &config).await?;
    for entry in &result.urls {
        println!("{}", entry.url);
    }
    ```

`MapResult.urls` is a list of `SitemapUrl` entries, each carrying `url` plus optional `lastmod`, `changefreq`, and `priority` when the source sitemap provides them.

## Headless-Browser Fallback

Static HTTP fetches miss content that only appears after JavaScript runs. `CrawlConfig.browser` (a `BrowserConfig`) controls when Xberg escalates to a headless browser. `BrowserMode` sets the policy:

| Mode      | Behaviour                                                                       |
| --------- | ------------------------------------------------------------------------------- |
| `Auto`    | Detect when JS rendering is needed and fall back to the browser (default).      |
| `Always`  | Route every request through the browser tier.                                   |
| `Never`   | Never use the browser fallback — static fetch only.                             |
| `Stealth` | Like `Always`, plus JS stealth patches, TLS fingerprint spoofing, and viewport override. |

=== "Python"

    ```python title="browser_fallback.py"
    from xberg import (
        BrowserConfig,
        BrowserMode,
        CrawlConfig,
        ExtractInput,
        ExtractionConfig,
        UrlExtractionConfig,
        extract,
    )

    config = ExtractionConfig(
        url=UrlExtractionConfig(
            crawl=CrawlConfig(
                browser=BrowserConfig(mode=BrowserMode.ALWAYS, wait_selector="main"),
            ),
        ),
    )
    output = await extract(
        ExtractInput(kind="uri", uri="https://spa.example.com"), config=config
    )
    print(output.results[0].content)
    ```

=== "TypeScript"

    ```typescript title="browser-fallback.ts"
    import { BrowserMode, ExtractInputKind, extract } from "@xberg-io/xberg";

    const output = await extract(
      { kind: ExtractInputKind.Uri, uri: "https://spa.example.com" },
      {
        url: {
          crawl: {
            browser: { mode: BrowserMode.Always, waitSelector: "main" },
          },
        },
      },
    );
    console.log(output.results[0].content);
    ```

=== "Rust"

    ```rust title="browser_fallback.rs"
    use xberg::{
        extract, BrowserConfig, BrowserMode, CrawlConfig, ExtractInput, ExtractionConfig, UrlExtractionConfig,
    };

    let config = ExtractionConfig {
        url: UrlExtractionConfig {
            crawl: CrawlConfig {
                browser: BrowserConfig {
                    mode: BrowserMode::Always,
                    wait_selector: Some("main".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    };
    let output = extract(ExtractInput::from_uri("https://spa.example.com"), &config).await?;
    println!("{}", output.results[0].content);
    ```

`BrowserConfig` also exposes `backend` (`Chromiumoxide` or `Native`), `wait` (`NetworkIdle`, `Selector`, `Fixed`), `timeout`, `extra_wait`, `block_url_patterns`, and `eval_script`. See the [BrowserConfig reference](../reference/configuration.md#browserconfig).

## Configuration Knobs

`UrlExtractionConfig` controls ingestion scope and safety:

| Field | Type | Default | Purpose |
| ----- | ---- | ------- | ------- |
| `mode` | `UrlExtractionMode` | `Auto` | Single-document fetch vs. crawl. |
| `crawl` | `CrawlConfig` | tuned defaults | Crawl depth, concurrency, robots, browser, proxy, auth. |
| `document_url_pattern` | `str \| None` | `None` | Regex filter for document-discovered URLs. |
| `max_document_urls_per_result` | `int \| None` | `100` | Cap on URLs followed per extraction result. |
| `max_total_urls` | `int \| None` | `1000` | Cap on URLs followed across the whole call. |
| `allow_local_file_inputs` | `bool` | `True` | Allow bare local filesystem path inputs. |
| `allow_file_uris` | `bool` | `True` | Allow local `file://` URI inputs. |

`CrawlConfig` defaults are tuned for polite, bounded crawls: `max_depth=1`, `max_pages=100`, `max_concurrent=10`, `respect_robots_txt=true`, `stay_on_domain=true`. Override them for wider or narrower runs. See the full [CrawlConfig reference](../reference/configuration.md#crawlconfig).

!!! warning "Untrusted URLs"
    Fetching user-supplied URLs is an SSRF surface. `CrawlConfig.ssrf` (an `SsrfPolicy`) governs which hosts and IP ranges are reachable. Keep `respect_robots_txt` and `stay_on_domain` enabled, and set explicit `max_pages` / `max_total_urls` bounds when crawling untrusted seeds.

## Related

- [Extraction Basics](extraction.md) — the shared `extract` / `extract_batch` entry points
- [Configuration Reference](../reference/configuration.md#urlextractionconfig) — `UrlExtractionConfig`, `CrawlConfig`, and `BrowserConfig` fields
- [crawlberg](https://github.com/xberg-io/crawlberg) — the standalone crawling and scraping engine behind URL ingestion
