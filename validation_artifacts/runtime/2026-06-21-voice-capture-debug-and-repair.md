# Voice Capture Debug And Repair

Scope: investigate Tree's report that a narrated Record & Replay dogfood run captured only two transcript words despite continuous speaking.

## Findings

- Source dogfood run inspected: `/private/tmp/narrated-record-replay/1782093012-review-the-gibson-nebuilder-lic-docx-report-and`.
- The run selected `Terry's AirPods Pro #3` from `macos-default-input`, sent Record & Replay events, and stopped cleanly, but packet inspection reported 1 transcript segment and 2 words.
- The Realtime transcription session must not use server VAD with `gpt-realtime-whisper`. A live smoke with server VAD returned `invalid_request_error` for `session.audio.input.turn_detection`.
- The correct client shape is manual audio commits. The helper now commits buffered audio every 5 seconds and skips empty final commits.
- The selected MacBook microphone path was quiet in local probing. Raw mic level probe without storing audio showed about `mean_volume: -51.7 dB`, `max_volume: -39.9 dB`.
- The helper now applies low-latency speech conditioning before streaming: `highpass=f=80,lowpass=f=9000,volume=18dB`.
- Filter probe without storing audio showed the conditioned path running at real-time speed with about `mean_volume: -37.8 dB`, `max_volume: -18.2 dB`.

## Changed Files

- `src/realtime/helpers.rs`
  - Keeps `turn_detection` null for `gpt-realtime-whisper`.
  - Adds a unit-tested manual commit predicate.
  - Counts delta transcript events distinctly.
- `src/realtime.rs`
  - Adds 5-second periodic manual `input_audio_buffer.commit` calls.
  - Skips final commit unless at least 100 ms of audio remains.
  - Adds capture counters to `post-commit-drain.json`.
  - Adds low-latency speech gain/filtering before PCM output.

## Verification

Commands run:

```sh
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cargo-target-check /Users/terrynoblin/personal-monorepo/.codex/skills/narrated-record-replay/scripts/check
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-plugin-cargo-target /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/scripts/check
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cache-cargo-target /Users/terrynoblin/.codex/plugins/cache/local-harness-plugins/narrated-record-replay/0.0.2-proposal/skills/narrated-record-replay/scripts/check
find /Users/terrynoblin/.codex/plugins/narrated-record-replay /Users/terrynoblin/.codex/plugins/cache/local-harness-plugins/narrated-record-replay/0.0.2-proposal /Users/terrynoblin/personal-monorepo/.codex/skills/narrated-record-replay -maxdepth 3 -type d \( -name target -o -name cargo-target \) -print
```

Results:

- Source skill: 46 Rust tests passed.
- Installed plugin: 46 Rust tests passed.
- Versioned cache: 46 Rust tests passed.
- No in-tree `target` or `cargo-target` directories were found.

Live smoke receipts:

- `/private/tmp/narrated-record-replay/1782093977-post-vad-fix-live-microphone-smoke`
  - Confirmed server rejects VAD for this transcription model.
- `/private/tmp/narrated-record-replay/1782094315-post-periodic-commit-live-microphone-smoke`
  - `audioCommitsSent: 3`, `audioBytesSent: 497120`, `realtimeErrorEventsObserved: 0`.
  - Produced empty transcript segments, consistent with no audible speech reaching the mic during the bounded window.
- `/private/tmp/narrated-record-replay/1782094568-deterministic-local-spoken-audio-microphone-smok`
  - Confirmed the selected MacBook mic path was open and committing, but local system speech was still too weak to produce transcript text before gain repair.

## Claim Ceiling

This repairs the API/session shape, periodic audio commit behavior, empty commit handling, app-visible plugin sync, and weak-input conditioning.

It does not prove full non-toy narrated Record & Replay usefulness. A follow-up dogfood still needs Tree speaking near the selected Mac or selected AirPods microphone and then packet/review inspection with non-sparse narration.

It also does not prove automatic Record & Replay lifecycle coupling. The bundled Record & Replay plugin exposes `event_stream_start`, `event_stream_status`, and `event_stream_stop` MCP tools, but no local lifecycle hook for the Rust helper to subscribe to. Until a wrapper/hook exists, R&R does not automatically stop when the mic helper reaches its bound.
