# 2026-06-22 Sample Spine And Delay Compare

## Scope

This receipt covers the non-live timing-spine repair before the next dogfood
run. It implements metadata surfaces needed to evaluate alignment from audio
sample position and to compare realtime delay runs mechanically.

## Implemented

- `audio-chunks.jsonl` records retained-audio chunk byte spans, sample spans,
  sample rate, and process-local monotonic capture offset.
- `narration.sync.jsonl` records narration capture start/stop sentinels with
  process-local monotonic offsets.
- `delay-compare` writes `delay-comparison.json` from high and low
  `delay-evaluation.json` artifacts.
- `delay-comparison.json` never authorizes a default-delay change by itself;
  operator review of transcript/action window usefulness remains required.
- Packet inspection classifies `audio-chunks.jsonl` and `narration.sync.jsonl`
  as raw-local private artifacts.
- Package hygiene rejects `audio-chunks.jsonl`, `narration.sync.jsonl`, and
  `delay-comparison.json`.

## Commands

```sh
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cargo-target cargo test --manifest-path .codex/skills/narrated-record-replay/Cargo.toml delay_compare
.codex/skills/narrated-record-replay/scripts/check-package-hygiene /private/tmp/nrr-package-hygiene-delay-compare-test
```

## Results

- Focused `delay_compare` tests: 2 passed, 0 failed.
- Synthetic package containing `delay-comparison.json`: rejected by package
  hygiene as expected.

## Claim Ceiling

This improves the next live proof path. It does not prove Record & Replay shares
the narration monotonic clock, does not prove drift correction, and does not
close `CLAIM-009`. Production-grade alignment still requires shared monotonic
timestamps, a muxed audio/video media stream, or start/stop sync sentinels on
both narration and Record & Replay sides.
