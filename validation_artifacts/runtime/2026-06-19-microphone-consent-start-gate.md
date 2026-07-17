---
last_edited: 2026-06-15
---

# Microphone Consent Start Gate

Date: 2026-06-19

Claim ID: `CLAIM-008`

## Claim Ceiling

This receipt proves the live `start` command fails closed unless the explicit
`--i-consent-to-microphone-capture` flag is present. It does not prove live
narrated capture, realtime transcription, packet usefulness, timeline
alignment, or replay behavior.

## Commands

Focused tests:

```sh
cargo fmt --manifest-path .codex/skills/narrated-record-replay/Cargo.toml -- --check
python3 -m pytest tests/test_narrated_record_replay_preflight.py
```

Result:

- `cargo fmt --check`: exit 0.
- `tests/test_narrated_record_replay_preflight.py`: 7 passed.

Manual no-consent start attempt:

```sh
cargo run --quiet --manifest-path .codex/skills/narrated-record-replay/Cargo.toml -- start --goal "consent gate proof" --root /tmp/nrr-consent-gate-proof-20260619T2340 --max-seconds 1
test ! -e /tmp/nrr-consent-gate-proof-20260619T2340
```

Result:

```text
--i-consent-to-microphone-capture is required before opening the microphone
```

The `test ! -e ...` check exited 0, proving the rejected command did not create
the capture root or spawn a capture session.

Preflight output now names the required flag:

```json
{
  "readyForLiveNarratedCapture": false,
  "recordReplayReady": true,
  "recordReplayStatus": {
    "confirmed": true,
    "source": "operator-provided-from-event-stream-status",
    "status": "idle"
  },
  "requiredConsentFlag": "--i-consent-to-microphone-capture",
  "opensMicrophone": false,
  "callsOpenAI": false,
  "blockers": [
    "explicit operator consent is required before opening the microphone"
  ]
}
```

## Result

Live narration capture is now mechanically guarded by an explicit CLI consent
flag. Future agents cannot open the microphone through `start` by only providing
a goal and key. The consent flag is documented in `SKILL.md` and surfaced by
preflight.

## Residual Risk

- The consent gate is a CLI guard, not OS-level permission enforcement.
- No microphone was opened in this proof.
- A live bounded capture still requires explicit operator consent, Record &
  Replay start/stop, realtime transcription output, packet generation,
  inspection, and receipt generation.
