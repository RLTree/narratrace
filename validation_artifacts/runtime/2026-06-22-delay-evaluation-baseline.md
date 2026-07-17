# 2026-06-22 Delay Evaluation Baseline

## Scope

This receipt covers the Rust-native delay evaluation artifact added for future
`high` versus `low` dogfood comparison. It does not change the default realtime
delay and does not prove `low` is acceptable.

## Commands

```sh
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cargo-target cargo test --manifest-path .codex/skills/narrated-record-replay/Cargo.toml delay_eval
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cargo-target cargo run --quiet --manifest-path .codex/skills/narrated-record-replay/Cargo.toml -- delay-eval --session-dir /private/tmp/narrated-record-replay/1782167614-0-0-7-aligned-transcript-video-dogfood-run --recording-metadata /var/folders/6n/9_g0pjg51lg2s6hnlnf1q8hh0000gn/T/sky/event_stream/851F87AB-603E-46E3-B90B-A7725344149B/session.json --recording-events /var/folders/6n/9_g0pjg51lg2s6hnlnf1q8hh0000gn/T/sky/event_stream/851F87AB-603E-46E3-B90B-A7725344149B/events.jsonl
```

## Results

- Focused `delay_eval` test: 1 passed, 0 failed.
- Delay evaluation artifact path:
  `/private/tmp/narrated-record-replay/1782167614-0-0-7-aligned-transcript-video-dogfood-run/delay-evaluation.json`.
- Baseline realtime delay: `high`.
- Record & Replay start to capture-clock audio start: `18,548 ms`.
- Record & Replay start to first audio chunk: `19,778 ms`.
- First realtime delta latency: `7,932 ms`.
- First completed realtime segment latency: `10,613 ms`.
- Final aligned utterance count: `17`.
- Unresolved mismatches: `0`.
- Diagnostic scripted marker recall: `17/17`.

## Claim Ceiling

This proves the comparison artifact exists and can extract privacy-safe timing
and count metrics plus local provenance metadata from an existing run. It does
not copy raw transcript text or audio, and it does not prove a paired `low` run,
transcript/action usefulness, operator review, or a default-delay change.
