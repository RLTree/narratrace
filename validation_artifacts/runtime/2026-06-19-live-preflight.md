---
last_edited: 2026-06-15
claim_ids: CLAIM-008
---

# Live Capture Preflight Receipt

## Purpose

Add a privacy-safe preflight surface for bounded live dogfood. The command reports readiness and blockers without starting Record & Replay, opening the microphone, calling OpenAI, or persisting raw transcript/audio.

## Command

```sh
cargo run --quiet --manifest-path .codex/skills/narrated-record-replay/Cargo.toml -- preflight --json --goal "demo workflow" --max-seconds 90
```

Observed output shape:

- `schema`: `narrated-record-replay.preflight.v1`
- `opensMicrophone`: `false`
- `callsOpenAI`: `false`
- `doesNotStartRecordReplay`: `true`
- `localPrerequisitesReady`: `true`
- `readyForLiveNarratedCapture`: `false`
- `recommendedCommand` includes `--max-seconds 90`
- `blockers` includes Record & Replay event-stream confirmation and explicit operator consent before microphone capture
- `claimCeiling`: preflight only; does not prove live Record & Replay, microphone transcription, packet usefulness, or replay behavior

The preflight reports only `hasOpenAIKey` as a boolean and does not print or store the key.

## Verification

Command:

```sh
.codex/skills/narrated-record-replay/scripts/check
```

Result:

- Exit code: `0`
- Cargo tests: `0 passed; 0 failed`
- Pytest: `24 passed`

## Claim Ceiling

This strengthens readiness evidence for `CLAIM-008`, but it does not close live narrated capture. `CLAIM-008` still requires an actual Record & Replay plus realtime narration run, generated packet artifacts, and inspection for raw-private leakage.
