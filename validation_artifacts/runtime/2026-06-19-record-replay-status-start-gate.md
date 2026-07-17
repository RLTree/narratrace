---
last_edited: 2026-06-15
---

# Record & Replay Status Start Gate

Date: 2026-06-19

Claim ID: `CLAIM-008`

## Claim Ceiling

This receipt proves the live `start` command fails closed unless the caller
passes an externally observed `--record-replay-status idle` value as well as
the microphone consent flag. It does not prove live narrated capture, realtime
transcription, packet usefulness, timeline alignment, or replay behavior.

## Commands

Focused tests:

```sh
cargo fmt --manifest-path .codex/skills/narrated-record-replay/Cargo.toml -- --check
python3 -m pytest tests/test_narrated_record_replay_preflight.py
```

Result:

- `cargo fmt --check`: initially required a mechanical wrap; `cargo fmt` was
  applied.
- `tests/test_narrated_record_replay_preflight.py`: 9 passed.

Manual missing-status start attempt:

```sh
cargo run --quiet --manifest-path .codex/skills/narrated-record-replay/Cargo.toml -- start --goal "record replay status gate proof" --root /tmp/nrr-rnr-status-gate-proof-20260619T2355 --max-seconds 1 --i-consent-to-microphone-capture
test ! -e /tmp/nrr-rnr-status-gate-proof-20260619T2355
```

Result:

```text
--record-replay-status idle is required before narrated capture
```

The root-existence check exited 0.

Manual active-recording status attempt:

```sh
cargo run --quiet --manifest-path .codex/skills/narrated-record-replay/Cargo.toml -- start --goal "record replay recording gate proof" --root /tmp/nrr-rnr-recording-gate-proof-20260619T2355 --max-seconds 1 --record-replay-status recording --i-consent-to-microphone-capture
test ! -e /tmp/nrr-rnr-recording-gate-proof-20260619T2355
```

Result:

```text
Record & Replay is already recording; stop it before narrated capture
```

The root-existence check exited 0.

## Result

`start` now requires the pre-live app status check to be carried into the CLI.
Future agents cannot open narrated capture with only microphone consent; they
must also pass `--record-replay-status idle`, sourced from the app-visible
Record & Replay status check.

## Residual Risk

- The status value is operator-provided to the CLI; it is not fetched directly
  by the Rust helper.
- No microphone was opened in this proof.
- No Record & Replay recording was started in this proof.
- Live bounded capture and generated packet proof remain owed.
