---
last_edited: 2026-06-15
---

# Clock Alignment Skew Fixture Receipt

Date: 2026-06-19

Scope: additional synthetic timestamp/alignment diagnostics for `CLAIM-009`.

## Command

```sh
.codex/skills/narrated-record-replay/scripts/check
```

Result: exit 0.

Pytest reported 9 passed.

## Covered Behavior

- `temporal-context.json` reports `recordReplayToAudioStartDeltaMs`.
- `clockSkewStatus` is `within-window` when Record & Replay start and audio start match within the configured window.
- `clockSkewStatus` is `exceeds-window` when Record & Replay metadata starts outside the configured window.
- `clockSkewWarningMs` is emitted so downstream validators can apply the same threshold.
- Repeated normalized transcript text is counted in `duplicateTranscriptSegments`.
- Duplicate details identify the repeated segment id, the first matching segment id, text, and transcript window.
- `docs/design-docs/temporal-context-contract.md` records the temporal context fields and current claim ceiling.

## Claim Ceiling

Supported:

- Synthetic fixture coverage for duplicate transcript text and explicit wall-clock skew diagnostics.
- A documented temporal-context contract for the current generated packet shape.

Unsupported:

- Live Record & Replay plus narration capture.
- Monotonic clock drift proof.
- Any claim that audio and UI timelines align correctly in real workflows.
