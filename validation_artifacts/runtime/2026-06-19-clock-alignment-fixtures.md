---
last_edited: 2026-06-15
---

# Clock Alignment Fixture Receipt

Date: 2026-06-19

Scope: synthetic timestamp/alignment diagnostics for `CLAIM-009`.

## Command

```sh
.codex/skills/narrated-record-replay/scripts/check
```

Result: exit 0.

Pytest reported 8 passed.

## Covered Behavior

- Temporal context emits `alignmentPolicy` with window size, confidence thresholds, clock assumptions, and monotonic-clock claim ceiling.
- Record & Replay events include `timestampParseStatus`.
- Malformed Record & Replay timestamps are reported in `alignmentDiagnostics.malformedRecordReplayTimestamps`.
- Missing event timestamps are counted in `alignmentDiagnostics.recordReplayEventsWithoutTimestamp`.
- Events outside the timestamp alignment window are counted in `alignmentDiagnostics.outOfWindowRecordReplayEvents`.
- Segment alignments include `alignmentMethod`, `alignmentWindowMs`, per-event confidence, segment confidence, and clock assumptions.
- Missing `audioStartedAtUnixMs` prevents alignment and records a `no alignment without audioStartedAtUnixMs` ceiling.

## Claim Ceiling

Supported:

- Synthetic fixture coverage for malformed timestamps, missing timestamps, out-of-window events, confidence labels, and missing audio anchor behavior.
- Static wall-clock alignment policy is explicit in generated temporal context.

Unsupported:

- Live Record & Replay plus narration capture.
- Monotonic clock drift proof.
- Replay-time behavior.
- Any claim that audio and UI timelines align correctly in real workflows.
