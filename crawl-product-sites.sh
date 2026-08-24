#!/usr/bin/env bash
# Crawl homepage of every product origin. Polite: sequential, 15s timeout.
set -euo pipefail
DEF="product-websites/scrape-definition.json"
OUT="$HOME/.spis/product-corpus"
mkdir -p "$OUT"
DONE=0; FAILED=0

for rec in $(jq -c '.records[]' "$DEF"); do
    origin=$(echo "$rec" | jq -r '.origin')
    name=$(echo "$rec" | jq -r '.product_name')
    slug=$(echo "$origin" | sed 's|https\?://||; s|[./:]|-|g; s|-$||')
    out_file="$OUT/$slug.jsonl.gz"
    [ -f "$out_file" ] && continue

    status=$(curl -sS --max-time 15 -A "WisentKronikaCorpus/0.1" -o /tmp/spis-page.html -w "%{http_code}" "$origin" 2>/dev/null) || { FAILED=$((FAILED+1)); continue; }
    size=$(stat -f%z /tmp/spis-page.html 2>/dev/null || echo 0)
    title=""
    if [ -s /tmp/spis-page.html ]; then
        title=$(sed -n 's/.*<title>\(.*\)<\/title>.*/\1/p' /tmp/spis-page.html | head -1)
    fi
    echo "{\"url\":\"$origin\",\"status\":$status,\"bytes\":$size,\"title\":$(printf '%s' "$title" | jq -Rs .),\"body_html\":$(jq -Rs . < /tmp/spis-page.html)}" | gzip > "$out_file"
    DONE=$((DONE+1))
    echo "  $slug: HTTP $status, ${size}B"
done

echo ""
echo "Done: $DONE crawled, $FAILED failed"
ls "$OUT/"/*.gz 2>/dev/null | wc -l
