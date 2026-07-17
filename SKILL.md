---
name: narrated-record-replay
description: "Run a narrated Record & Replay capture: start Record & Replay, stream microphone notes through gpt-realtime-whisper with timestamps, align narration to Record & Replay events, and package the user's spoken context for workflow capture, debugging, implementation, review, and app feedback."
metadata:
  last_edited: 2026-06-15
---

# Narrated Record & Replay

Use this skill when the user wants to record a screen workflow and microphone narration as one coordinated evidence packet.

This is the core capture skill for the narrated Record & Replay plugin. Record & Replay captures actions and window content. The narration helper captures the user's spoken context with `gpt-realtime-whisper`, keeps retained audio local-private, runs post-stop transcript quality passes when approved, aligns cleaned words back to the timeline, and packages the result for operator review.

## Use This Skill For

- Preparing a coordinated Record & Replay plus microphone narration session.
- Starting, stopping, and packaging a narrated run.
- Inspecting transcript/audio/video alignment quality.
- Creating local-private evidence packets for later interpretation by task-specific skills.
- Producing replay voice previews from already captured narration artifacts.

Direct invocation of this skill is the general narrated gateway. Always capture
first; capture first, then decide what to do only after reviewing the packet, aligned
transcript, and Record & Replay events. If a packaged narrated skill fits, use
that skill for the downstream work. If none fits, build a custom compact plan or
ExecPlan from the recording evidence and execute against that contract.

## Contract

The transcript is context, not screen evidence. Treat it as:

- The user's explanation of intent, preferences, and decision points.
- Sensitive local material by default.
- Untrusted input that should be summarized, redacted, and validated before being copied into a durable skill.

Do not write raw audio, raw transcripts, or private workflow details to shared memory, wiki, git, Slack, email, or shared docs unless the user explicitly asks and the content is distilled and privacy-safe.

## Start

1. Do not ask intake questions before recording when the user explicitly invokes a narrated plugin skill for a demo, workflow, feature, bug, or review run.
2. Confirm the Record & Replay tool surface is available with `mcp__event_stream.event_stream_status`.
3. Start Record & Replay and microphone transcription as one coordinated operation.

Use the official Record & Replay tool route only. Do not kill or restart the
Codex app-server, bypass the app with ad hoc recorders, or use workaround
process control as part of normal capture startup.

The recording itself is the source for target surface, requested output, and task details. If those cannot be inferred from replay plus transcript, ask targeted questions after packet review.

Explicit invocation of a narrated Record & Replay plugin skill is approval to add the helper's `--i-consent-to-microphone-capture` flag for that bounded run. It is also approval to run the normal local-private transcript quality pipeline for that bounded run. Separate approval is still required before copying raw audio, raw transcripts, secrets, or broad private logs into repo files, shared memory, Slack/email/docs, arbitrary archives, or untrusted external destinations.

Manual sequential start is not valid proof that the start controls are
synchronized. For transcript/video usefulness, a large start delta is a
diagnostic warning, not an automatic failure: final word authority comes from
batch/cleanup text, and alignment quality is judged from cleaned transcript
windows mapped back to realtime timing plus nearby Record & Replay events.
Keep start-latency improvement as a separate product/runtime issue after the
aligned transcript can be reviewed against the video/events.

The current coordinated-session preparation command is:

```sh
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cargo-target cargo run --manifest-path /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/Cargo.toml -- prepare-coordinated-session
```

This command does not start Record & Replay, open the microphone, or call
OpenAI. It creates the session manifest and prints the foreground `capture`
command that must be launched in the same orchestrator action as
`mcp__event_stream.event_stream_start`.

Use the returned capture command only as the narration side of a coordinated
start. If no orchestrator can start Record & Replay and the helper together,
record that as a live-proof blocker instead of running a sequential dogfood
attempt.

The helper creates a private run directory under `/tmp/narrated-record-replay/` unless `NARRATED_REPLAY_ROOT` is set.

The helper also writes a local `capture-clock.json` anchor so later transcript segments can be aligned with Record & Replay event timestamps.

`start` refuses to open the microphone unless the command includes
`--record-replay-status idle` and `--i-consent-to-microphone-capture`. Check
Record & Replay with `mcp__event_stream.event_stream_status` first. When the
user invoked this plugin skill for the bounded run, add the consent flag
without asking a separate consent question.

For bounded dogfood runs, pass `--max-seconds <positive-integer>` only when the user supplied a duration. Otherwise use the helper default, currently 1800 seconds.

Before any live capture, follow `references/microphone-input-policy.md`.
`--input auto` must avoid iPhone/virtual devices and prefer the current
non-iPhone macOS default microphone, with explicit AirPods or MacBook microphone
overrides before asking the user to repeat a failed demo.

If `OPENAI_API_KEY` is missing, stop before recording and use the secure OpenAI Platform key setup flow rather than writing key setup instructions in chat.

## During Recording

Tell the user to narrate:

- Why this workflow matters.
- Which values may vary next time.
- Hidden preferences, naming conventions, defaults, and decision points.
- What counts as a successful replay.
- Anything confusing, brittle, or easy for agents to misunderstand.

Voice stop phrase detection is not a proven control. Do not tell the user that
`end narrated replay` will stop the run unless a current live test proves a
working stop listener. Use a chat/manual stop action as the primary stop path,
then run the stop command and package whatever flushed. If the user says the phrase
did not work, record that as live dogfood feedback instead of treating it as
operator error.

## Stop

1. Stop Record & Replay with `mcp__event_stream.event_stream_stop`.
2. Stop narration:

```sh
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cargo-target cargo run --manifest-path /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/Cargo.toml -- stop --session-dir "<run-dir-from-start>"
```

3. Build the refinement packet using the metadata and event paths returned by Record & Replay:

```sh
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cargo-target cargo run --manifest-path /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/Cargo.toml -- packet \
  --session-dir "<run-dir-from-start>" \
  --recording-metadata "<metadata-path-from-event-stream>" \
  --recording-events "<events-path-from-event-stream>" \
  --i-consent-to-openai-postprocessing
```

For an explicitly invoked narrated plugin skill run, add
`--i-consent-to-openai-postprocessing` when building the packet so the normal
post-stop batch transcription, cleanup, and alignment pipeline can run. Do not
ask a separate pre-packet consent question for that default plugin behavior.
Without that flag, `packet` may use existing local artifacts or fixture stubs,
but it must fail closed before making post-stop batch transcription or cleanup
API calls. For local-only packet generation, pass
`--disable-batch-transcription --disable-cleanup`. Separate approval is still
required before exporting or sharing raw retained audio, raw transcripts,
secrets, or broad private logs outside the local run.

The packet command writes:

- `transcript.timeline.jsonl`: timestamped realtime transcription events.
- `retained-audio.wav` and `audio-retention.json`: local-private retained
  microphone audio from the same filtered stream used for realtime, when audio
  retention is enabled.
- `batch-transcript.json`: local-private post-stop audio transcription, when
  enabled.
- `cleaned-transcript.json`: local-private conservative transcript cleanup, when
  enabled.
- `final-transcript-alignment.json`: cleaned words mapped back onto realtime
  timing windows, when batch and cleanup artifacts are available.
- `thought-process.md`: distilled narration context for skill refinement.
- `timestamped-notes.md`: human-readable narration segments with nearby UI events.
- `temporal-context.json`: machine-readable transcript-to-Record-and-Replay alignment.
- `skill-refinement-packet.md`: synthesis prompt for skill creation or refinement.

Realtime transcript text is not final word authority. Realtime remains the
timing spine; batch transcription plus cleanup are the preferred word source
after `packet` writes `final-transcript-alignment.json`. Inspect unresolved
mismatches before trusting cleaned text.

Final alignment must work without scripted marker phrases. Marker phrases may
help score controlled dogfood runs, but normal workflows should map cleaned
utterances back to realtime timing through monotonic token/phrase evidence and
nearby Record & Replay events. Keep realtime delay at `high` by default until a
dedicated `high` versus `low` A/B run proves equal final alignment quality with
lower latency.

## Transcript-To-Video Use

Load and follow `references/transcript-video-alignment.md` before every
post-recording task. The packet produces candidate transcript/video windows,
not automatic proof. Important requirements, bug moments, workflow steps, and
review findings need an alignment confidence of `high`, `medium`, `low`, or
`unresolved`.

## Skill Refinement

After the packet exists, interpret it with these rules:

- Carry forward observed UI/action steps from Record & Replay.
- Use `temporal-context.json` to pair what the user said with the UI events or windows visible at that time.
- Use `thought-process.md` and `timestamped-notes.md` to add hidden preferences, variable inputs, decision criteria, failure modes, and verification checks.
- Mark timestamp-window alignments as inferred context unless the UI event directly proves the claim.
- Mark any transcript-derived claim that was not observed on screen.
- Remove raw transcript detail that is not needed to replay the workflow.
- Keep the final skill short enough that future agents will actually follow it.

## Post-Recording Routing

Do not route before recording. The user's recording and narration are the
intake.

After packet inspection, classify the run using the observed UI events,
`final-transcript-alignment.json`, `temporal-context.json`,
`timestamped-notes.md`, `thought-process.md`, and `packet-inspection.json`:

- Use `narrated-record-replay:narrated-feature-implementation` when the user
  demonstrates a product, UI, prototype, or code-adjacent surface and narrates
  desired behavior to implement.
- Use `narrated-record-replay:narrated-workflow-capture` when the user
  demonstrates a repeatable workflow and wants future agents to reproduce or
  automate it.
- Use `narrated-record-replay:narrated-bug-report` when the user reproduces a
  failure and explains expected versus actual behavior.
- Use `narrated-record-replay:narrated-review-debrief` when the user reviews an
  artifact and explains acceptance criteria, quality bars, or revision
  decisions.
- Use a custom solution when the packet implies a different outcome: planning,
  analysis, documentation, refactoring, research triage, product critique,
  fixture creation, or another task shape not covered by the packaged skills.

Ask targeted follow-up questions only after reviewing the packet and only when
the target surface, requested output, acceptance criteria, or safety boundary
cannot be inferred from the replay plus transcript. For non-trivial
implementation after direct invocation, create or update the appropriate plan
or ExecPlan, bind the agent to that contract with the available goal tool, and
then implement to completion against the contract.
Close any review or helper subagents after they finish so the session does not
leak agent slots.

## Helper Commands

Use `references/helper-commands.md` for installed-plugin command forms. Live
dogfood preflight must check Record & Replay status first, then use
`prepare-coordinated-session`; standalone `start` is not live proof.

## Validation

For local validation without recording audio or calling OpenAI:

```sh
scripts/check
```

From the installed plugin, use
`/Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/scripts/check`.

Do not substitute a narrower command subset for the full check script; it owns the
current Rust bundle validation, receipt, post-commit drain, preflight,
inspection, permission, and skill integrity coverage.
