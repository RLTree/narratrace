# Default Mic Empty Completion Smoke

Date: 2026-06-21

Scope: patched plugin smoke for default microphone selection, clean stop, empty completed transcript handling, and optional replay-plan inspection.

Private run: `/private/tmp/narrated-record-replay/1782019206-patched-empty-completion-and-default-mic-smoke`

Record & Replay session used for packaging exercise: `66519D30-837F-4B10-9A02-95A80D436E9D`

## Findings

- Live AVFoundation device listing exposed `Terry's iPhone Microphone` at audio index `0` and `MacBook Pro Microphone` at audio index `1`.
- macOS default input resolved to `MacBook Pro Microphone`.
- `--input auto` selected `MacBook Pro Microphone` with `ffmpegInput=:1` and source `macos-default-input`.
- The capture helper reached `state=stopped` without a stop timeout.
- Realtime returned completion messages with empty transcript text during the smoke. Empty completed events must not count as transcript segments.
- The generated artifact leak scan must not block on a missing `replay-voice-execution-plan.json` when no replay voice preview was generated.
- Final review HTML must be included in generated artifact leak scanning because operators inspect or share that artifact.
- Missing or malformed replay voice execution plans must keep the review contract blocked rather than receiving proof credit for mere file existence.

## Repairs

- `src/realtime/helpers.rs` now ignores empty completed transcription events for segment accounting and does not write blank final transcript or timeline rows.
- `src/inspect.rs` includes `replay-voice-execution-plan.json` in generated leak scanning only when that file exists.
- `src/inspect.rs` rescans generated artifacts after `review-artifact.html` is rendered and includes the final HTML review artifact in the scan set.
- `src/review/contract.rs` blocks review status when the replay voice dry-run plan is missing or malformed.
- `src/bundle/util.rs` has a bundle-specific symlink-ancestor regression for validator read/hash paths.
- `src/session/preflight.rs` exposes an audio input preview before microphone capture.
- `src/review/html.rs` surfaces helper stop state, selected audio input, post-commit transcript segment count, and stop-timeout state.
- Focused tests cover empty completed transcript events and omitted optional generated artifacts.

## Verification

- `CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-plugin-cargo-target /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/scripts/check`: exit 0, 39 Rust tests passed.
- `CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cargo-target-check /Users/terrynoblin/personal-monorepo/.codex/skills/narrated-record-replay/scripts/check`: exit 0, 39 Rust tests passed.
- `CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-plugin-cargo-target cargo run --quiet --manifest-path /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/Cargo.toml -- capture --session-dir /private/tmp/narrated-record-replay/1782019206-patched-empty-completion-and-default-mic-smoke --delay low --input auto --max-seconds 12 --record-replay-status idle --i-consent-to-microphone-capture`: exit 0.
- `status.json`: `state=stopped`, `deviceName=MacBook Pro Microphone`, `ffmpegInput=:1`, `source=macos-default-input`.
- `post-commit-drain.json`: `completedSegments=0`, `messages=4`, `errors=[]`, `waitedMs=5002`.
- `packet-inspection.json`: generated artifact leak scan status `expected-local-references-only`, artifact count `8`, `review-artifact` scanned, blocking findings `0`.
- `review-contract.json`: status `blocked`, replay voice preview status `not-generated`, replay voice preview plan valid `false`.
- `dogfood-receipt.json`: status `blocked`; blockers are same-start timestamp proximity, no transcript segments, no post-commit completed transcript segments, and required operator review.

## Artifact Digests

- `status.json`: `sha256:5e10e0a48c231b2f133adc293342bd70693db5ee8c0064c26113993f8339f74d`
- `post-commit-drain.json`: `sha256:b2ab74e54a3ed1fce1e4d589484c182dfa749874990f0f3b8771a52de1dc2940`
- `parent-operation-receipt.json`: `sha256:b522f521aff1af898797d94eaf321fb4a95f8e0f0225495116ea8e9d71a4b1c4`
- `packet-inspection.json`: `sha256:fb0e71e3bf5c4510638db996527c7c7f50c133d52883de17eb8975d8fc593afa`
- `review-contract.json`: `sha256:17abd4422d848bb6d42d1c09dbb2aaeb4218be70f661dd9f0f9c4dea88e7478f`
- `review-artifact.html`: `sha256:6f2bd3639bdbe9d61455fba93bf34e8f0acd9302207169d513681d66ad6f9396`
- `dogfood-receipt.json`: `sha256:2e57eeb1d07c3d2d15bd06a39003afdaf0f9fce36d20733c502f08c5eb4338c5`

## Claim Ceiling

This smoke proves default microphone selection, non-iPhone refusal coverage by tests, clean helper stop, corrected empty-transcript accounting, final review artifact leak scanning, and fail-closed missing replay preview status. It does not prove coordinated same-operation Record & Replay plus microphone start, non-empty transcription usefulness, operator review, or replay-time voice execution.
