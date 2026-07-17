---
last_edited: 2026-06-15
claim_ids: CLAIM-008
---

# Bounded Capture Guard Receipt

## Purpose

Make live narrated dogfood safer by allowing `start` to pass `--max-seconds <positive-integer>` through to the internal `capture` process. The capture loop stops automatically after the bound if an explicit stop action is missed.

## Changed Surface

- `src/config.rs`: parses and validates `--max-seconds`.
- `src/session.rs`: passes the bound to the background `capture` process.
- `src/realtime.rs`: checks elapsed capture time and includes `maxSeconds` in status JSON.
- `SKILL.md`: documents bounded dogfood usage.
- `tests/test_narrated_record_replay.py`: covers help text and invalid values.

## Verification

Command:

```sh
.codex/skills/narrated-record-replay/scripts/check
```

Result:

- Exit code: `0`
- Cargo tests: `0 passed; 0 failed`
- Pytest: `18 passed`

## Claim Ceiling

This supports only a safety prerequisite for `CLAIM-008`. It does not prove microphone capture, realtime transcription, usable transcript text, live alignment, packet usefulness, or redaction completeness.
