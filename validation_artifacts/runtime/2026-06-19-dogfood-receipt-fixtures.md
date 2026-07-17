---
last_edited: 2026-06-15
created_at: 2026-06-19T18:10:00Z
claim_ids: CLAIM-008, CLAIM-010
artifact_type: runtime-fixture-receipt
---

# Dogfood Receipt Fixtures

## Purpose

Add a privacy-safe `receipt` command for live dogfood runs. The command writes
`dogfood-receipt.json` with artifact paths, existence, byte sizes, line counts,
and SHA-256 content fingerprints. It does not copy raw transcript text or audio
into the receipt.

## Changed Surface

- `src/receipt.rs`: new dogfood receipt compiler.
- `src/receipt/artifacts.rs`: artifact metadata and fingerprint helpers.
- `src/main.rs`: routes the `receipt` command.
- `src/config.rs`: documents `receipt` in helper usage.
- `SKILL.md`: adds the receipt command to the skill workflow.
- `tests/test_narrated_record_replay.py`: verifies a secret-bearing synthetic
  transcript does not leak raw text into the receipt.

## Verification

- `cargo fmt --manifest-path .codex/skills/narrated-record-replay/Cargo.toml`
  - Exit: `0`.
- `python3 -m pytest tests/test_narrated_record_replay.py`
  - Exit: `0`.
  - Result: `20 passed`.

## Claim Ceiling

This strengthens the live proof path for `CLAIM-008` and the privacy boundary
for `CLAIM-010`. It does not close either claim. A real bounded dogfood capture,
packet generation, packet inspection, receipt generation, and operator review
are still required.
