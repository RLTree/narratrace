---
last_edited: 2026-06-15
---

# Reliability

Reliability means a narrated capture can be stopped, inspected, resumed, rejected, or packaged without losing evidence boundaries.

## State Roots

- Default private run root: `/tmp/narrated-record-replay/`.
- Override: `NARRATED_REPLAY_ROOT`.
- Worktree-local generated state: `.codex-worktree/` after sourcing `.codex-worktree/env.sh`.
- Durable receipts: `validation_artifacts/`.

## Recovery Rules

- `start` writes a manifest and status before capture work.
- `capture` writes `capture-clock.json` before transcript events.
- `stop` creates `.stop` and waits for `status.json` to reach a terminal state.
- `packet` must tolerate missing optional Record & Replay artifacts by lowering claim ceiling, not faking alignment.

## Reliability Gaps To Close

- Add tests for malformed timestamps, missing clock anchors, out-of-window events, duplicate transcript events, partial captures, and interrupted ffmpeg/websocket sessions.
- Define bounded retry/timeout behavior for realtime network failures before live-use claims.
- Preserve failure receipts under `validation_artifacts/` without dumping raw transcripts.
