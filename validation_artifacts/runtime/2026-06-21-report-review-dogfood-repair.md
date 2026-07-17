# Report Review Dogfood Repair

Date: 2026-06-21

Scope: plugin dogfood run `019ee862-d9f2-71b0-9ed9-f469f26134b6` for report artifact review.

Private run: `/private/tmp/narrated-record-replay/1782016380-review-the-gibson-nebuilder-lic-markdown-report`

Record & Replay session: `274E1621-6591-4AE2-B697-083A42031493`

## Findings

- Voice stop phrase detection is not proven. Tree reported that `end narrated replay` did not stop the run.
- The capture helper left `status.json` at `recording` after manual stop and forced termination.
- No `post-commit-drain.json` was written because the helper was terminated before clean drain.
- `dogfood-receipt.json` treated a blocked parent-operation receipt as missing timestamp proximity, even though `startDeltaMs=3046` and `withinAllowedStartDelta=true`.
- `packet-inspection.json` treated missing optional `transcript.final.txt` as `unsafe-artifact-path`, which made the raw-local sensitive summary misleading.

## Repairs

- `SKILL.md` now says voice stop phrase detection is not a proven control and chat/manual stop is primary.
- `src/realtime.rs` checks `.stop` at the top of the capture loop before competing audio/read branches.
- `src/session.rs` writes `stop-requested` status while waiting and emits `stop-timeout.json` if the helper does not reach `stopped` or `failed`.
- `src/receipt.rs` exposes parent-operation, capture, review, generated leak scan, raw-local private, and raw-local sensitive surfaces directly in `dogfood-receipt.json`.
- `src/receipt.rs` no longer labels matching timestamp proximity as missing just because the parent receipt status is blocked by clean-shutdown or drain failures.
- `src/inspect/artifacts.rs` no longer treats a missing optional raw-local artifact as sensitive; symlinks and unsafe existing paths still block.

## Verification

- `python3 /Users/terrynoblin/.codex/skills/.system/plugin-creator/scripts/validate_plugin.py /Users/terrynoblin/.codex/plugins/narrated-record-replay`: exit 0.
- `CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-plugin-cargo-target /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/scripts/check`: exit 0, 34 Rust tests passed.
- `CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cargo-target-check /Users/terrynoblin/personal-monorepo/.codex/skills/narrated-record-replay/scripts/check`: exit 0, 34 Rust tests passed.
- Refreshed `packet-inspection.json`: `transcript-final` now has `exists=false`, `containsSensitivePatterns=false`, and no sensitive categories.
- Refreshed `dogfood-receipt.json`: status remains `blocked`, with blockers limited to dirty helper stop, missing post-commit drain receipt, and required operator review.

## Refreshed Artifact Digests

- `packet-inspection.json`: `sha256:d3bc8da1e21a3f69be46e0e631dc2b2fdd6441722dc7e7ef204dec726b7af83b`
- `dogfood-receipt.json`: `sha256:068facfa18409a5aee56f8f1647502ed18590b3f64667d09aee9683450f55d47`

## Claim Ceiling

This repair improves shutdown reliability and proof accounting. It does not prove a clean live capture. Full live capture claims still require a new bounded dogfood run with clean helper stop, `post-commit-drain.json`, current packet inspection, current receipt, and operator review.
