---
last_edited: 2026-06-15
---

# Preflight Gated Start Command

Date: 2026-06-19

Claim ID: `CLAIM-008`

## Claim Ceiling

This receipt proves preflight now emits a start command that includes the
required Record & Replay idle-status gate and microphone-consent gate while
still marking explicit consent as required. It does not prove consent, live
capture, realtime transcription, packet usefulness, timeline alignment, or
replay behavior.

## Commands

```sh
cargo fmt --manifest-path .codex/skills/narrated-record-replay/Cargo.toml -- --check
python3 -m pytest tests/test_narrated_record_replay_preflight.py
cargo run --quiet --manifest-path .codex/skills/narrated-record-replay/Cargo.toml -- preflight --json --goal "narrated Record & Replay ultragoal live dogfood" --max-seconds 120 --record-replay-status idle
```

Results:

- `cargo fmt --check`: exit 0.
- `tests/test_narrated_record_replay_preflight.py`: 9 passed.
- Preflight command: exit 0.

Preflight excerpt:

```json
{
  "readyForLiveNarratedCapture": false,
  "recordReplayReady": true,
  "recommendedCommand": "cargo run --manifest-path .codex/skills/narrated-record-replay/Cargo.toml -- start --goal \"narrated Record & Replay ultragoal live dogfood\" --max-seconds 120 --record-replay-status idle --i-consent-to-microphone-capture",
  "recommendedCommandRequiresExplicitConsent": true,
  "requiredConsentFlag": "--i-consent-to-microphone-capture",
  "opensMicrophone": false,
  "callsOpenAI": false,
  "blockers": [
    "explicit operator consent is required before opening the microphone"
  ]
}
```

## Result

The preflight receipt no longer suggests a stale `start` command. It now points
to the mechanically gated command and explicitly labels it as requiring consent
before use.

## Residual Risk

- No microphone was opened.
- No Record & Replay recording was started.
- The preflight command is still a local readiness receipt, not live narrated
  capture proof.
