---
last_edited: 2026-06-15
created_at: 2026-06-19T19:35:00Z
claim_ids: CLAIM-010
artifact_type: fixture-validation-receipt
---

# Email Punctuation Redaction Fixtures

## Purpose

Close a redaction edge case where email-like identifiers followed by common
sentence punctuation were not detected by generated-artifact redaction or
raw-local category-only inspection.

## Failing Case Proved First

The focused tests were changed to put synthetic email values at sentence end.
Before the redaction patch:

- `tests/test_narrated_record_replay.py::test_generated_artifacts_redact_secret_like_transcript_text`
  failed because `tree@example.com.` remained in generated packet text.
- `tests/test_narrated_record_replay_inspect.py::test_inspect_packet_summarizes_raw_local_sensitive_categories_without_values`
  failed because the raw-local category summary omitted `email`.

## Change

`looks_like_email` now trims common trailing sentence punctuation before
classification. The generated replacement still preserves punctuation around
the redacted marker through the existing edge-punctuation preservation path.

## Verification

Command:

```sh
cargo fmt --manifest-path .codex/skills/narrated-record-replay/Cargo.toml -- --check
```

Result: exit 0.

Command:

```sh
python3 -m pytest tests/test_narrated_record_replay.py::test_generated_artifacts_redact_secret_like_transcript_text
```

Result: exit 0, `1 passed`.

Command:

```sh
python3 -m pytest tests/test_narrated_record_replay_inspect.py::test_inspect_packet_summarizes_raw_local_sensitive_categories_without_values
```

Result: exit 0, `1 passed`.

Command:

```sh
.codex/skills/narrated-record-replay/scripts/check
```

Result: exit 0. Cargo test harness completed successfully and pytest reported
`31 passed`.

## Claim Ceiling

This improves `CLAIM-010` fixture coverage for generated redaction and
raw-local category inspection. It does not prove full privacy safety, live
narrated capture privacy, or real non-toy packet privacy review.
