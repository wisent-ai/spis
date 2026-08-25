# Concept: crawled documentation corpus

The **docs corpus** is a resumable full-text archive of the documentation-site inventory. Inventory definitions live in `documentation-site-examples/content-structure/`; fetched state lives under `$HOME/.spis/docs-corpus/<site-slug>/`.

It is separate from the interface-reference catalogs. A documentation-site record in a catalog describes an interface; the docs corpus stores extracted pages for read-only search and inspection.

## Stored files

| File | Contents |
|---|---|
| `pages.jsonl.gz` | append-only gzip members containing one JSON page per line |
| `state.json` | URL-hash map with URL and observed status; periodically written through `state.json.tmp` then renamed |
| `done.json` | scraper resume marker used by product-page scraping where applicable |

A page record includes URL, fetch time, HTTP status, quality classification, body hash/size, optional title/text, and sitemap `lastmod`. HTML is converted to text. High brace density is classified as `css_js_noise`; its text is withheld.

## Acquisition

`crawl-docs` resolves inventories sequentially, then uses a flat worker queue across selected sites. Defaults are 64 workers and a 0.3-second per-host delay. Robots rules are checked and cached by origin. HTTP attempts have a 45-second deadline and retry 429 plus selected 5xx responses; gzip bodies are detected by magic bytes.

One writer thread per site serializes gzip output. A state flusher snapshots progress every ten seconds so a long crawl can resume. Final state uses a temporary file and rename.

## Read-only JSON interface

```bash
spis docs-corpus status
spis docs-corpus search --query <text> [--site <slug>] [--limit <n>]
spis docs-corpus show --site <slug> --url <exact-url>
```

`status` combines checked-in inventory metadata with local state. `search` scans decompressed records and returns `hits`, `scanned`, and `limit`; the current implementation accepts `--site` but scans the collected site list without applying that filter. `show` emits the exact matching JSON record. A concurrently appended gzip member may end mid-stream; readers return everything that decompressed cleanly and stop at the tear.

With an empty temporary `HOME`, an executed `status` still listed the 50 checked-in inventories with `seen: 0`; `search --query install` returned `{"hits":[],"scanned":0,"limit":20}`. That makes the query surface safe to inspect before any crawl has populated local pages.
