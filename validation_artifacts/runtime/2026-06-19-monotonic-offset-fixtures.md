---
last_edited: 2026-06-15
claim_ids: CLAIM-009
---

# Monotonic Offset Fixture Receipt

## Purpose

Strengthen the clock contract by capturing process-local monotonic transcript offsets during live narration and surfacing those offsets in generated temporal context.

## Changed Surface

- `src/realtime.rs`: passes elapsed `Instant` time into transcript event recording.
- `src/timeline/transcript.rs`: writes `monotonicOffsetMs` into raw local transcript events when provided and exposes timing source in transcript segments.
- `src/timeline.rs`: reports `alignmentPolicy.monotonicClock.status`, monotonic segment counts, and conservative claim ceiling.
- `docs/design-docs/temporal-context-contract.md`: documents process-local monotonic offsets and their limits.
- `tests/test_narrated_record_replay.py`: covers temporal context with `process-local-offsets-captured`.

## Verification

Command:

```sh
.codex/skills/narrated-record-replay/scripts/check
```

Result:

- Exit code: `0`
- Cargo tests: `0 passed; 0 failed`
- Pytest: `21 passed`

## Claim Ceiling

This strengthens fixture-level evidence for `CLAIM-009`. It does not prove cross-process monotonic drift between Record & Replay and audio capture. `CLAIM-009` remains blocked until a live capture demonstrates clock anchors, alignment windows, confidence labels, and drift assumptions against real Record & Replay artifacts.
