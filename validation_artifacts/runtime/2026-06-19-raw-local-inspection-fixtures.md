---
last_edited: 2026-06-15
created_at: 2026-06-19T19:20:00Z
claim_ids: CLAIM-010
artifact_type: fixture-validation-receipt
---

# Raw Local Inspection Fixtures

## Purpose

Harden `packet-inspection.json` so raw local transcript artifacts are summarized
without copying raw transcript text or matched sensitive values into durable
inspection artifacts.

## Change

- `privacyBoundary.rawLocalOnly` now reports byte counts, line counts, SHA-256
  content fingerprints, category-only sensitive-pattern flags, and sensitive
  categories for raw local transcript artifacts.
- The inspection artifact continues to mark raw transcript artifacts as
  `raw-local-private`.
- Generated artifact leak scanning remains separate under
  `privacyBoundary.generatedArtifactLeakScan`.

## Verification

Command:

```sh
cargo fmt --manifest-path .codex/skills/narrated-record-replay/Cargo.toml -- --check
```

Result: exit 0.

Command:

```sh
python3 -m pytest tests/test_narrated_record_replay_inspect.py
```

Result: exit 0, `6 passed`.

Command:

```sh
.codex/skills/narrated-record-replay/scripts/check
```

Result: exit 0. Cargo test harness completed successfully and pytest reported
`31 passed`.

## Privacy Assertion

The regression fixture writes a synthetic raw transcript line containing a
secret-like token, sensitive key/value phrase, and email-like identifier. The
test asserts that `packet-inspection.json` contains category names and
fingerprint metadata, but not the raw secret or email value.

## Claim Ceiling

This improves `CLAIM-010` fixture coverage and operator inspection metadata.
It does not prove full redaction safety, live narrated capture privacy, or
real non-toy packet privacy review.
