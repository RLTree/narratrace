---
last_edited: 2026-07-17
---

# Architecture

This directory is a repo-local Codex skill plus a Rust helper CLI for narrated Record & Replay capture.

## Bird's-Eye View

`SKILL.md` is the discoverable skill entrypoint. The Rust helper captures microphone transcription through `gpt-realtime-whisper`, records local timestamp anchors, aligns transcript segments with Record & Replay event artifacts, and writes skill-refinement packets. The current helper proves fixture-level packet generation only; live UI plus microphone capture still needs dogfood proof.

## Layer Boundaries

- Skill contract: `SKILL.md`, `AGENTS.md`, `AGENT_STANDARDS.md`, `GOAL_CONTRACT.md`, and this routed setup surface.
- CLI boundary: `src/config.rs` parses arguments and environment-derived defaults.
- Session boundary: `src/session.rs` owns local run directories, manifests, status, start, stop, and validate behavior.
- Realtime boundary: `src/realtime.rs` owns websocket transcription and ffmpeg audio capture.
- Model-input boundary: batch transcription and cleanup keep static trusted policy separate from JSON-encoded untrusted audio/transcript evidence and reuse only receipt-bound artifacts.
- Timeline boundary: `src/timeline.rs` and `src/timeline/` own transcript timestamps, Record & Replay event parsing, typed agent-facing evidence, and receipt-gated alignment authority.
- Packet boundary: `src/packet.rs` owns generated refinement artifacts and evidence-boundary language.
- Proof boundary: bundle, parent-operation, receipt, and review modules parse closed schemas and withhold positive states when authenticated goal/run authority is unavailable.
- Validation boundary: `scripts/check`, `VALIDATION.md`, Rust bundle validation, and Cargo checks.

## Codemap

```text
SKILL.md                     discoverable Codex skill instructions
AGENTS.md                    local router for future agents
AGENT_STANDARDS.md           skill-local law from the harness plugin proposal
GOAL_CONTRACT.md             claim ids, proof surfaces, and completion ceiling
EXECPLAN.md                  current living plan
src/                         Rust CLI helper implementation
scripts/check                setup and helper validation gate
docs/                        routed design, plan, generated, product, and reference indexes
validation_artifacts/        receipts and generated proof, not raw transcripts
.codex/                      skill-local Codex worktree environment setup
```

## Invariants Stated As Absences

- No raw audio, raw transcripts, credentials, or broad private logs belong in durable skill files.
- No transcript-derived claim is screen evidence unless Record & Replay directly proves it.
- No UI/review/product claim can close from CLI proof alone.
- No live-use claim can close from synthetic fixtures alone.
- No future lane may write outside its owned paths without a contract update.

## Where To Change Things

- CLI behavior: edit `src/`, then run `scripts/check` and add focused tests.
- User-facing skill instructions: edit `SKILL.md`, keep it concise, and preserve discoverability.
- Standards, scope, or claim ceiling: update `AGENT_STANDARDS.md`, `GOAL_CONTRACT.md`, `EXECPLAN.md`, and backlog rows together.
- Runtime/dogfood receipts: write distilled receipts under `validation_artifacts/`; keep raw local run outputs outside durable docs unless explicitly approved and redacted.
