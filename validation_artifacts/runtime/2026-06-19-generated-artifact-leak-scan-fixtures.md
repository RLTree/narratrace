---
last_edited: 2026-06-15
claim_ids: CLAIM-010, CLAIM-011, CLAIM-012
---

# Generated Artifact Leak Scan Fixture Receipt

## Purpose

Make packet inspection actively scan generated review candidates for obvious
unredacted sensitive patterns. The scan reports only artifact names and
categories, not matched sensitive values.

## Changed Surface

- `src/redaction.rs`: exposes category-only sensitive pattern detection using
  the same pattern family as generated-artifact redaction.
- `src/inspect.rs`: adds
  `privacyBoundary.generatedArtifactLeakScan` and a blocker when generated
  review candidates contain obvious unredacted sensitive categories.
- `tests/test_narrated_record_replay_inspect.py`: adds a fixture that appends a
  synthetic secret to a generated packet and verifies the inspection reports
  `secret-token` without copying the secret value.
- `docs/design-docs/redaction-policy.md`: documents the category-only scanner
  and its claim ceiling.

## Focused Verification

Commands:

```sh
cargo fmt --manifest-path .codex/skills/narrated-record-replay/Cargo.toml
python3 -m pytest tests/test_narrated_record_replay_inspect.py
.codex/skills/narrated-record-replay/scripts/check
```

Results:

- `cargo fmt`: exit code `0`
- `pytest tests/test_narrated_record_replay_inspect.py`: `3 passed`
- `scripts/check`: exit code `0`; Cargo tests `0 passed; 0 failed`;
  pytest `27 passed`

## Claim Ceiling

This strengthens fixture-level privacy and evidence-boundary inspection for
`CLAIM-010`, `CLAIM-011`, and `CLAIM-012`. It still does not close those claims:
the scan is pattern-based, real non-toy packet inspection is still owed, and
operator review is still required before sharing generated artifacts.
