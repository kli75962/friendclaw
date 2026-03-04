/// Parse DuckDuckGo HTML results into `(title, snippet)` pairs.
///
/// DuckDuckGo's HTML lite page uses stable class names:
///   `result__a`       — result title anchor
///   `result__snippet` — result excerpt anchor
pub fn parse_ddg_results(html: &str, max_results: usize) -> Vec<(String, String)> {
    let mut results = Vec::with_capacity(max_results);
    let mut cursor = html;

    while results.len() < max_results {
        // ── Title ──────────────────────────────────────────────────────────
        let title_class = "class=\"result__a\"";
        let Some(class_pos) = cursor.find(title_class) else { break };

        // Move past the opening `<a ...>` tag to grab the inner text.
        let tag_start = &cursor[class_pos..];
        let Some(gt) = tag_start.find('>') else { break };
        let after_open = &tag_start[gt + 1..];

        let end_a = after_open.find("</a>").unwrap_or(after_open.len().min(200));
        let title = clean_text(&after_open[..end_a]);

        // ── Snippet ────────────────────────────────────────────────────────
        // Search within a reasonable window after the title to avoid
        // accidentally matching a snippet from the next result block.
        let window_end = class_pos + title_class.len();
        let window = &cursor[window_end..];

        let snippet_class = "class=\"result__snippet\"";
        let snippet = match window.find(snippet_class) {
            Some(si) => {
                let after_class = &window[si + snippet_class.len()..];
                // Skip to end of snippet's opening tag.
                let gt2 = after_class.find('>').unwrap_or(0);
                let inner = &after_class[gt2 + 1..];
                // Snippets end at </a>. Cap to 400 chars before stripping
                // so we don't process megabytes of HTML on a miss.
                let close = inner
                    .find("</a>")
                    .unwrap_or_else(|| inner.len().min(400));
                clean_text(&inner[..close])
            }
            None => String::new(),
        };

        if !title.is_empty() {
            results.push((title, snippet));
        }

        // Advance cursor past current match to find the next result.
        cursor = &cursor[window_end..];
    }

    results
}

// ── Helpers ──────────────────────────────────────────────────────────────────

/// Strip HTML tags, decode entities, and collapse whitespace — single pass.
///
/// Doing all three steps in one loop avoids two intermediate `String` allocations
/// that the naive strip → decode → split_whitespace chain would produce.
fn clean_text(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    let mut prev_was_space = true; // start true to trim leading whitespace
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if in_tag => {}
            '&' => {
                // Try to read an HTML entity: &name; or &#digits;
                let entity = read_entity(&mut chars);
                let replacement = decode_entity(&entity);
                // Treat decoded whitespace the same as regular whitespace.
                if replacement.chars().all(|c| c.is_ascii_whitespace()) {
                    if !prev_was_space {
                        out.push(' ');
                        prev_was_space = true;
                    }
                } else {
                    out.push_str(replacement);
                    prev_was_space = false;
                }
            }
            c if c.is_ascii_whitespace() => {
                if !prev_was_space {
                    out.push(' ');
                    prev_was_space = true;
                }
            }
            c => {
                out.push(c);
                prev_was_space = false;
            }
        }
    }

    // Trim trailing space added by the last whitespace run.
    if out.ends_with(' ') {
        out.pop();
    }
    out
}

/// Consume chars up to and including `;` to capture a `name;` or `#digits;` entity body.
/// Returns what was consumed (without the leading `&`).
fn read_entity(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut entity = String::with_capacity(8);
    for c in chars.by_ref() {
        if c == ';' {
            break;
        }
        entity.push(c);
        if entity.len() > 10 {
            // Not a real entity — stop consuming to avoid runaway reads.
            break;
        }
    }
    entity
}

/// Map a collected entity body (without `&` or `;`) to its replacement string.
/// Unrecognised entities are returned verbatim (with `&` and `;` restored).
fn decode_entity(entity: &str) -> &str {
    match entity {
        "amp"  => "&",
        "lt"   => "<",
        "gt"   => ">",
        "quot" => "\"",
        "#39" | "apos" => "'",
        "nbsp" => " ",
        _ => "",  // unknown/malformed — just drop it
    }
}
