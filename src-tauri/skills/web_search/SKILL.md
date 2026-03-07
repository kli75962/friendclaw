---
name: web_search
description: Search DuckDuckGo for real-time information in HK (e.g weather, date).
compatibility: PhoneClaw (Tauri v2 Android agent)
---

## Usage

```
web_search(query: "concise keyword query", max_results: 5)
```

- Use **keywords**, not full sentences. Good: `"duckduckgo privacy settings 2025"`. Bad: `"What are the privacy settings for duckduckgo in 2025?"`.
- `max_results` defaults to 5. Use 3 for quick lookups, up to 10 for broad research.
- Results are numbered: title on the first line, snippet indented below.

---

## Result Format

```
1. Page Title
   Short excerpt describing the page content.

2. Another Result
   Another excerpt...
```

Extract the relevant answer from the snippets. If the snippets are insufficient, search again with a refined query.

---

## Rules

- Always **cite what you found** when answering the user — don't present search results as your own knowledge.
- If the first search is unhelpful, **refine the query** (add a year, remove ambiguous words, try synonyms).
- Do not call `web_search` more than **15 times** per user request to avoid unnecessary delay.
```
