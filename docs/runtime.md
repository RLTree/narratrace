---
last_edited: 2026-06-15
---

# Runtime

This skill owns a local Rust CLI helper.

## Commands

```sh
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cargo-target cargo run --manifest-path /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/Cargo.toml -- help
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cargo-target cargo run --manifest-path /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/Cargo.toml -- validate --json
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cargo-target cargo run --manifest-path /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/Cargo.toml -- packet --session-dir <dir> --i-consent-to-openai-postprocessing
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cargo-target cargo run --manifest-path /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/Cargo.toml -- delay-eval --session-dir <dir> --recording-metadata <metadata-path> --recording-events <events-path>
```

## State

- Private capture root: `/tmp/narrated-record-replay/` unless `NARRATED_REPLAY_ROOT` is set.
- Worktree-local state after setup: `.codex-worktree/state/`.
- Cargo target after setup: `/private/tmp/narrated-record-replay-cargo-target/`.
- Durable receipts: `validation_artifacts/`.

## Transcription Quality Pipeline

Normal packet generation now uses realtime transcription as the timing spine and
post-stop artifacts as the word-quality spine.

- Post-stop OpenAI calls are gated by packet-time intent:
  `--i-consent-to-openai-postprocessing`. For a user-invoked narrated plugin or
  skill run, invocation is the current bounded approval to add that helper flag
  for the normal local-private batch transcription, cleanup, and alignment
  pipeline. Without it, `packet` can consume existing local artifacts or
  fixtures, but it fails closed before new batch transcription or cleanup API
  calls.
- Realtime default delay: `high`. Override with
  `NARRATED_REPLAY_REALTIME_DELAY` or `--delay minimal|low|medium|high|xhigh`.
- Batch transcription default: enabled with `gpt-4o-transcribe`. Override with
  `NARRATED_REPLAY_BATCH_MODEL`, `--batch-transcription-model`, or disable with
  `NARRATED_REPLAY_BATCH_TRANSCRIPTION=0` / `--disable-batch-transcription`.
- Cleanup default: enabled with `gpt-5.4-mini`, falling back to `gpt-5-mini` when
  the default model is unavailable. Override with `NARRATED_REPLAY_CLEANUP_MODEL`
  or `--cleanup-model`; disable with `NARRATED_REPLAY_CLEANUP=0` /
  `--disable-cleanup`.
- Model vocabulary is a static public allowlist. Session metadata, event text,
  workflow terms, and caller-provided dictionary files are not sent to the
  transcription or cleanup models.
- Audio retention default: `private-wav`, writing the same filtered 24 kHz mono
  PCM microphone stream used for realtime to local-private runtime artifacts.
  Override with `NARRATED_REPLAY_AUDIO_RETENTION_MODE` /
  `--audio-retention-mode` and `NARRATED_REPLAY_AUDIO_RETENTION_PATH` /
  `--audio-retention-path`.
- Retained audio now has metadata-only timing companions:
  `audio-chunks.jsonl` records per-chunk byte spans, `sampleStart`,
  `sampleEnd`, and process-local monotonic capture offset; `narration.sync.jsonl`
  records start/stop sentinels. These are local-private timing metadata, not
  raw audio or transcript content.
- Audio filter default:
  `highpass=f=80,lowpass=f=9000,volume=9dB,alimiter=limit=0.95`.
  Override with `NARRATED_REPLAY_AUDIO_FILTER` or `--audio-filter`. The limiter
  is intentional: the earlier `volume=18dB` path clipped AirPods/default-mic
  dogfood audio and produced poor post-stop transcription.
- Final transcript alignment uses cleaned batch text as word authority only
  after a current v2 receipt binds the session, batch, cleanup response,
  realtime timeline, final artifacts, and conservative-transform policy.
  Missing, disabled, malformed, stale, or digest-mismatched receipts fall back
  to realtime text. Marker phrases remain optional scripted-test anchors.
- Keep realtime delay default at `high` until scripted dogfood proves that
  `low` preserves final aligned transcript quality. Delay changes are timing
  and anchor-quality tuning, not the primary fix for word authority.

## Delay Evaluation Policy

Do not switch normal dogfood to `low` just because startup feels slower. Compare
`high` and `low` only after final transcript/video reconciliation is healthy.
The A/B receipt should compare:

- first audio chunk delta from Record & Replay start,
- first realtime delta latency,
- first completed realtime segment latency,
- anchor/phrase recall across the cleaned utterances,
- final aligned utterance count,
- unresolved mismatches,
- transcript/action window usefulness.

Raw realtime word accuracy is diagnostic only. The main metric is final aligned
transcript quality against the video/event windows.

Use `delay-eval` after each scripted dogfood run to write
`delay-evaluation.json`. The artifact records timing/count metrics plus local
provenance metadata such as artifact paths and audio-input metadata:
first audio chunk delta, first realtime delta latency, first completed
realtime segment latency, final aligned utterance count, unresolved mismatches,
Record & Replay event count, and diagnostic scripted marker recall. Marker
recall is for controlled scoring only and must not become a product alignment
dependency.

Use `delay-compare` after both `high` and `low` runs have
`delay-evaluation.json` artifacts:

```sh
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cargo-target cargo run --manifest-path /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/Cargo.toml -- delay-compare --session-dir <comparison-dir> --baseline-delay-evaluation <high-delay-evaluation.json> --candidate-delay-evaluation <low-delay-evaluation.json>
```

`delay-comparison.json` may report that a candidate has lower latency, but it
never authorizes a default-delay change by itself. Operator review of final
transcript/action window usefulness is still required.

Current `high` baseline from private dogfood session
`/private/tmp/narrated-record-replay/1782167614-0-0-7-aligned-transcript-video-dogfood-run`:

- Record & Replay start to capture-clock audio start: `18,548 ms`.
- Record & Replay start to first audio chunk: `19,778 ms`.
- First realtime delta latency: `7,932 ms`.
- First completed realtime segment latency: `10,613 ms`.
- Final aligned utterance count: `17`.
- Unresolved mismatches: `0`.
- Scripted marker recall: `17/17`, diagnostic only.

Private runtime artifacts include `retained-audio.wav`, `audio-retention.json`,
`audio-chunks.jsonl`, `narration.sync.jsonl`, `batch-transcript.json`,
`batch-transcription-receipt.json`, `cleaned-transcript.json`,
`cleanup-receipt.json`, `final-transcript-alignment.json`,
`final-transcript.timeline.jsonl`, `delay-evaluation.json`, and
`delay-comparison.json`.
Generated packets link to these artifacts for local review but do not embed raw
audio or raw transcript content by default.

## Logs And Receipts

The default validation gate writes `validate --json` output to the process temp directory. Future live dogfood runs should copy only redacted, distilled receipts into `validation_artifacts/`.

## Runtime Claim Ceiling

`validate --json` proves dependency detection and model-name wiring only.
Fixture tests prove the local batch/cleanup/final-alignment contracts without
opening a microphone or calling OpenAI. The full `scripts/check` gate now also
requires authenticated host goal/run attestation before it can refresh positive
bundle proof; caller-controlled environment or JSON observations fail closed.
No local fixture proves real capture, OpenAI availability, Record & Replay
integration, packet usefulness, or UI/video alignment quality.
