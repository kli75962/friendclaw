use reqwest::Client;
use std::sync::OnceLock;
use std::time::Duration;

use super::parser::parse_ddg_results;

/// A single shared HTTP client for all web searches.
/// Reusing it preserves the connection pool, avoiding repeated TCP+TLS handshakes.
static CLIENT: OnceLock<Client> = OnceLock::new();

fn client() -> &'static Client {
    CLIENT.get_or_init(|| {
        Client::builder()
            .user_agent(
                "Mozilla/5.0 (Linux; Android 10; Mobile) AppleWebKit/537.36 \
                 (KHTML, like Gecko) Chrome/120.0.0.0 Mobile Safari/537.36",
            )
            .timeout(Duration::from_secs(12))
            .build()
            .expect("failed to build reqwest Client")
    })
}

/// Execute a DuckDuckGo HTML search and return top results as plain text.
///
/// Uses the lite HTML endpoint — no JavaScript, no API key needed.
/// Output is capped to `max_results` to stay within LLM token budget.
pub async fn web_search(query: &str, max_results: usize) -> String {
    let resp = match client()
        .get("https://html.duckduckgo.com/html/")
        .query(&[("q", query), ("kl", "us-en")])
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => return format!("error: search request failed: {e}"),
    };

    if !resp.status().is_success() {
        return format!("error: DuckDuckGo returned HTTP {}", resp.status());
    }

    let html = match resp.text().await {
        Ok(t) => t,
        Err(e) => return format!("error: failed to read response body: {e}"),
    };

    eprintln!("[web_search] raw HTML (first 3000 chars):\n{}", &html[..html.len().min(3000)]);

    let results = parse_ddg_results(&html, max_results);

    if results.is_empty() {
        return "No results found.".to_string();
    }

    // Pre-estimate capacity: title (~60) + snippet (~200) + formatting per result.
    let mut out = String::with_capacity(results.len() * 280);
    for (i, (title, snippet)) in results.iter().enumerate() {
        if i > 0 {
            out.push_str("\n\n");
        }
        use std::fmt::Write;
        if snippet.is_empty() {
            let _ = write!(out, "{}. {}", i + 1, title);
        } else {
            let _ = write!(out, "{}. {}\n   {}", i + 1, title, snippet);
        }
    }
    out
}
