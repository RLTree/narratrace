---
last_edited: 2026-06-15
created_at: 2026-06-19T20:05:00Z
claim_ids: CLAIM-010, CLAIM-012
artifact_type: browser-runtime-receipt
---

# Review Raw-Local Category Browser Proof

## Purpose

Prove the static review page renders raw-local sensitive category summaries in
browser runtime without exposing matched raw transcript values.

## Synthetic Source

Session directory:

```text
/tmp/nrr-review-raw-local-category-browser-proof-20260619T2005/session
```

The synthetic transcript contained a secret-like token, sensitive key/value
phrase, and email-like identifier. The raw values are intentionally omitted from
this durable receipt.

## Commands

Packet generation:

```sh
cargo run --quiet --manifest-path .codex/skills/narrated-record-replay/Cargo.toml -- packet --session-dir /tmp/nrr-review-raw-local-category-browser-proof-20260619T2005/session
```

Result: exit 0.

Inspection and review refresh:

```sh
cargo run --quiet --manifest-path .codex/skills/narrated-record-replay/Cargo.toml -- inspect --session-dir /tmp/nrr-review-raw-local-category-browser-proof-20260619T2005/session
```

Result: exit 0. It wrote `packet-inspection.json`, `review-contract.json`, and
`review-artifact.html`.

Static server:

```sh
python3 -m http.server 8781 --bind 127.0.0.1
```

Result: `GET /review-artifact.html` returned HTTP 200. `favicon.ico` returned
404, which is not relevant to the artifact.

Browser proof:

```text
mcp__playwright.browser_navigate http://127.0.0.1:8781/review-artifact.html
mcp__playwright.browser_take_screenshot fullPage=true filename=narrated-review-raw-local-categories-20260619.png
mcp__playwright.browser_evaluate raw-local category checks
```

Observed:

```json
{
  "title": "Narrated Record & Replay Review",
  "hasRawLocalCount": true,
  "hasRawLocalCategoriesLine": true,
  "hasRecoveryAction": true,
  "hasChecklistItem": true,
  "hasSecretTokenCategory": true,
  "hasSensitiveKeyCategory": true,
  "hasEmailCategory": true,
  "leaksSecret": false,
  "leaksEmail": false,
  "bodyLength": 2429
}
```

## Claim Ceiling

This proves browser-runtime rendering for the raw-local sensitive category
summary on a synthetic local review artifact. It does not prove live narrated
capture, real non-toy packet review, operator product-cohesion approval, or
full privacy safety.
