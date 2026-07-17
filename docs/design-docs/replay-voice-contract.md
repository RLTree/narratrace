---
last_edited: 2026-06-15
---

# Replay Voice Contract

`replay-voice-parameters.json` is a planning artifact for future replay behavior.
It is not proof that a replay engine exists or that replay voice controls ran.

## Current Schema

Current schema id: `narrated-record-replay.replay-voice-parameters.v1`

Required top-level fields:

- `schema`: contract version.
- `status`: currently `planned-not-executed`.
- `claimCeiling`: explicit unsupported replay behavior statement.
- `defaults`: requested `style`, `pace`, and `emphasis`.
- `allowedValues`: accepted values for each voice parameter.
- `timelineBindingContract`: how future replay code must interpret segment bindings.
- `proofObligations`: evidence required before `CLAIM-013` can close.
- `segmentBindings`: one binding per temporal transcript segment.

## Timeline Binding

Each binding points back to `temporal-context.transcriptSegments[]` and uses the
`transcript-audio-offset-ms` clock domain. These offsets are suitable for a
future replay engine only after the engine maps them through
`temporal-context.json` and records a replay execution receipt.

## Preview Plan

`replay-voice-preview` consumes `replay-voice-parameters.json` and writes
`replay-voice-execution-plan.json`. The preview proves deterministic scheduling
and parameter consumption only. Its status is `dry-run-not-spoken`; it must not
be used as proof that audio playback or live replay occurred.

## Claim Ceiling

The current contract supports typed replay voice planning plus deterministic
dry-run scheduling. `CLAIM-013` remains blocked until replay behavior tests and
a live demonstration prove that style, pace, and emphasis affect actual replay
behavior.
