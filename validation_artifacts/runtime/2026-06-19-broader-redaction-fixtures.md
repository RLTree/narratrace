---
last_edited: 2026-06-15
claim_ids: CLAIM-010
---

# Broader Redaction Fixture Receipt

## Purpose

Extend generated-artifact redaction beyond obvious secret tokens and email-like identifiers. The fixture covers synthetic local private paths, phone-like numbers, JSON Web Token-shaped values, and long opaque token-like values.

## Changed Surface

- `src/redaction.rs`: adds pattern redaction for private-looking local paths, phone-like numbers, JWT-shaped values, and long opaque tokens.
- `tests/test_narrated_record_replay.py`: verifies generated artifacts omit the synthetic private values while raw local transcript input remains unchanged.
- `docs/design-docs/redaction-policy.md`: documents the broadened pattern scope and remaining claim ceiling.

## Verification

Command:

```sh
.codex/skills/narrated-record-replay/scripts/check
```

Result:

- Exit code: `0`
- Cargo tests: `0 passed; 0 failed`
- Pytest: `19 passed`

## Claim Ceiling

This strengthens fixture-level evidence for `CLAIM-010`, but it is still pattern redaction only. `CLAIM-010` remains blocked until a real non-toy generated packet is inspected for necessary distilled content and raw-private leakage.
