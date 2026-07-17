---
last_edited: 2026-06-15
claim_ids: CLAIM-008
---

# Record & Replay Event-Stream Smoke Receipt

## Purpose

Prove the Record & Replay event-stream plugin surface is callable from this Codex session without starting microphone capture, realtime transcription, or persisting raw transcript/audio.

## Commands And Tool Calls

- `tool_search` query: `event-stream Record Replay status start stop`.
- `mcp__event_stream.event_stream_status`: returned idle status with `isRecording: false` and `maxDurationSeconds: 1800`.
- `mcp__event_stream.event_stream_start`: started session `99D74DD8-9E6C-449E-AE5B-B29A27A44FFE` at `2026-06-19T07:31:43Z`.
- `mcp__event_stream.event_stream_stop`: stopped the same session at `2026-06-19T07:31:47Z` with `endReason: tool_stopped`.

## Artifact Shape

- Metadata path: `/var/folders/6n/9_g0pjg51lg2s6hnlnf1q8hh0000gn/T/sky/event_stream/99D74DD8-9E6C-449E-AE5B-B29A27A44FFE/session.json`
- Metadata digest: `sha256:bb2eee0240a47d8b92033792cb258f835f76b82469f077f488289d37fb2b2a6a`
- Metadata top-level keys: `endReason,endedAt,eventsPath,id,startedAt`
- Events path: `/var/folders/6n/9_g0pjg51lg2s6hnlnf1q8hh0000gn/T/sky/event_stream/99D74DD8-9E6C-449E-AE5B-B29A27A44FFE/events.jsonl`
- Events digest: `sha256:d78185d5d3999f7fbb2cb8d9b4e45fc8952156dad161ac80d5baadd83737ebd2`
- Events row count: `3`
- Events top-level key shapes observed: `id,kind,timestamp`; `app,ax,id,kind,timestamp,window`

## Privacy Handling

Only file existence, row count, top-level keys, session id, timestamps, and SHA-256 digests were inspected or persisted. Raw event payload values, screenshots, audio, transcript text, window contents, and broad logs were not copied into this receipt.

## Claim Ceiling

This supports only a narrow prerequisite for `CLAIM-008`: the Record & Replay plugin can start, stop, and produce metadata/events artifacts in this session.

It does not prove live narrated capture, microphone capture, realtime Whisper transcription, audio/UI timestamp alignment, packet usefulness, redaction completeness, or replay voice behavior.
