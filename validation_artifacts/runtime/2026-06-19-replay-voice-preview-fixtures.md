---
last_edited: 2026-06-15
---

# Replay Voice Preview Fixtures

## Scope

This receipt covers a non-live `CLAIM-013` slice. It adds a deterministic
`replay-voice-preview` command that consumes `replay-voice-parameters.json` and
writes `replay-voice-execution-plan.json`.

This is a dry-run scheduling preview. It does not speak audio, run an actual
replay engine, prove live replay behavior, or close `CLAIM-013`.

## Manual Synthetic Proof

Synthetic session root:

```text
/tmp/nrr-replay-voice-preview-proof-20260619T2130/session
```

Packet command:

```sh
cargo run --quiet --manifest-path .codex/skills/narrated-record-replay/Cargo.toml -- packet --session-dir /tmp/nrr-replay-voice-preview-proof-20260619T2130/session --replay-voice-style calm --replay-voice-pace slow --replay-voice-emphasis high
```

Result: exit 0.

Preview command:

```sh
cargo run --quiet --manifest-path .codex/skills/narrated-record-replay/Cargo.toml -- replay-voice-preview --session-dir /tmp/nrr-replay-voice-preview-proof-20260619T2130/session
```

Result: exit 0.

Output summary:

```json
{
  "cueCount": 1,
  "replayVoiceExecutionPlanPath": "/tmp/nrr-replay-voice-preview-proof-20260619T2130/session/replay-voice-execution-plan.json",
  "sessionDir": "/tmp/nrr-replay-voice-preview-proof-20260619T2130/session",
  "status": "dry-run-not-spoken"
}
```

Plan inspection:

```json
{
  "schema": "narrated-record-replay.replay-voice-execution-plan.v1",
  "status": "dry-run-not-spoken",
  "cueCount": 1,
  "speaksAudio": false,
  "plannedDurationMs": 2500,
  "paceMultiplier": 0.8,
  "emphasisGain": 1.2,
  "tone": "steady and low-variance"
}
```

## Automated Proof

```sh
python3 -m pytest /Users/terrynoblin/personal-monorepo/tests/test_narrated_record_replay.py::test_packet_accepts_custom_replay_voice_parameters /Users/terrynoblin/personal-monorepo/tests/test_narrated_record_replay.py::test_replay_voice_preview_proves_parameter_dependent_scheduling /Users/terrynoblin/personal-monorepo/tests/test_narrated_record_replay.py::test_replay_voice_preview_requires_voice_artifact
```

Result: exit 0, `3 passed`.

```sh
.codex/skills/narrated-record-replay/scripts/check
```

Result: exit 0. Cargo tests passed with `0 passed`; pytest passed with
`33 passed`.

## Claim Ceiling

- Supports: deterministic preview scheduling consumes typed replay voice
  parameters, and tests prove `style`, `pace`, and `emphasis` change the dry-run
  plan.
- Does not support: audio playback, actual replay behavior, live replay, or the
  live demonstration required for `CLAIM-013`.
