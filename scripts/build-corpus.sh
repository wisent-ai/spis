#!/usr/bin/env bash
# build-corpus.sh — extract ALL reference records from ALL Spis catalogs
# into a flat markdown corpus suitable for AI consumption.
#
# Output: ~/.spis/writing-corpus/<catalog>/<NN-slug>.md per record,
#         plus an index.md per catalog listing all products.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
OUT="$HOME/.spis/writing-corpus"
mkdir -p "$OUT"

CATALOGS=(
  ios-app-examples android-app-examples macos-app-examples
  desktop-app-examples web-app-examples dashboard-console-examples
  tui-examples cli-examples onboarding-auth-examples
  documentation-site-examples app-store-listing-examples
  design-system-examples report-evidence-examples
  readme-examples pricing-page-examples landing-page-examples
)

TOTAL=0

for cat_dir in "${CATALOGS[@]}"; do
  ref_dir="$ROOT/$cat_dir/references"
  out_cat="$OUT/$cat_dir"
  mkdir -p "$out_cat"
  [ -d "$ref_dir" ] || continue

  # Count records
  COUNT=0
  for rec_dir in "$ref_dir"/*/; do
    [ -f "$rec_dir/reference.json" ] && COUNT=$((COUNT + 1))
  done

  # Catalog-level index
  {
    echo "# $cat_dir"
    echo ""
    echo "Records: $COUNT"
    echo ""
  } > "$out_cat/index.md"

  # Extract records
  for rec_dir in "$ref_dir"/*/; do
    [ -d "$rec_dir" ] || continue
    slug=$(basename "$rec_dir")
    ref_json="$rec_dir/reference.json"
    [ -f "$ref_json" ] || continue

    md_file="$out_cat/$slug.md"

    {
      echo "---"
      echo "catalog: $cat_dir"
      echo "slug: $slug"
      echo "---"
      echo ""

      name=$(jq -r '.name // empty' "$ref_json")
      purl=$(jq -r '.product_url // .source_url // empty' "$ref_json")
      echo "# $name"
      [ -n "$purl" ] && echo "URL: $purl"
      echo ""

      desc=$(jq -r '.description // empty' "$ref_json")
      [ -n "$desc" ] && { echo "$desc"; echo ""; }

      goal=$(jq -r '.journey.goal // empty' "$ref_json")
      actor=$(jq -r '.journey.actor // empty' "$ref_json")
      [ -n "$goal" ] && { echo "## Journey"; echo "**Goal:** $goal"; [ -n "$actor" ] && echo "**Actor:** $actor"; echo ""; }

      if command -v jq >/dev/null && jq -e '.interactions' "$ref_json" >/dev/null 2>&1; then
        ninter=$(jq '.interactions | length' "$ref_json")
        j=0
        while [ $j -lt $ninter ]; do
          iname=$(jq -r ".interactions[$j].name // \"step-$((j+1))\"" "$ref_json")
          trigger=$(jq -r ".interactions[$j].trigger // empty" "$ref_json")
          response=$(jq -r ".interactions[$j].response // empty" "$ref_json")
          feedback=$(jq -r ".interactions[$j].feedback // empty" "$ref_json")
          failure=$(jq -r ".interactions[$j].failure // empty" "$ref_json")
          recovery=$(jq -r ".interactions[$j].recovery // empty" "$ref_json")

          echo "### $iname"
          [ -n "$trigger" ] && echo "Trigger: $trigger"
          [ -n "$response" ] && echo "Response: $response"
          [ -n "$feedback" ] && echo "Feedback: $feedback"
          [ -n "$failure" ] && echo "Failure: $failure"
          [ -n "$recovery" ] && echo "Recovery: $recovery"
          echo ""
          j=$((j + 1))
        done
      fi

      acc_obs=$(jq -r 'if .accessibility then (.accessibility | tostring) else empty end // empty' "$ref_json" 2>/dev/null)
      [ -n "$acc_obs" ] && [ "$acc_obs" != "null" ] && {
        echo "## Accessibility observations"
        echo "$acc_obs"
        echo ""
      }

      ev_status=$(jq -r '.evidence_status // empty' "$ref_json" 2>/dev/null)
      [ -n "$ev_status" ] && { echo "**Evidence status:** $ev_status"; echo ""; }
    } > "$md_file"

    TOTAL=$((TOTAL + 1))
  done

  echo "$cat_dir → $COUNT records extracted"
done

echo ""
echo "Total records extracted: $TOTAL"
