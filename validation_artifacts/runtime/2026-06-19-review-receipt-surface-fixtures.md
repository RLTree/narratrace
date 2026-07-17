---
last_edited: 2026-06-15
created_at: 2026-06-19T18:25:00Z
claim_ids: CLAIM-012
artifact_type: runtime-fixture-receipt
---

# Review Receipt Surface Fixtures

## Purpose

Surface `dogfood-receipt.json` in the static review artifacts so live-dogfood
proof metadata is visible during operator review. The review HTML and
`review-contract.json` now report receipt presence and receipt status without
copying raw transcript text.

## Changed Surface

- `src/review.rs`: includes dogfood receipt path and status in review HTML.
- `src/review/contract.rs`: records `artifactPresence.dogfoodReceipt` and
  `reviewState.dogfoodReceiptStatus`.
- `src/packet.rs`: passes no receipt during initial packet generation.
- `src/inspect.rs`: passes an existing dogfood receipt to refreshed review
  artifacts when present.
- `tests/test_narrated_record_replay.py`: verifies review artifacts surface
  receipt status and do not expose a synthetic raw transcript secret.

## Verification

- `cargo fmt --manifest-path .codex/skills/narrated-record-replay/Cargo.toml`
  - Exit: `0`.
- `python3 -m pytest tests/test_narrated_record_replay.py tests/test_narrated_record_replay_inspect.py`
  - Exit: `0`.
  - Result: `25 passed`.
- `.codex/skills/narrated-record-replay/scripts/check`
  - Exit: `0`.
  - Result: Cargo harness clean; `30 passed`.

## Claim Ceiling

This strengthens fixture-level review contract coverage for `CLAIM-012`. It
does not close `CLAIM-012`: real non-toy packet review, browser/UI proof, and
operator product-cohesion review are still owed.
