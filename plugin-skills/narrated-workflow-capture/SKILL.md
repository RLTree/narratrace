---
name: narrated-workflow-capture
description: Use when the user wants to demonstrate an existing workflow with Record & Replay plus spoken narration so Codex can turn observed UI steps, decision points, defaults, and verification cues into reusable agent instructions, a workflow, a runbook, or an automation contract.
---

# Narrated Workflow Capture

Use this skill when the user wants a repeatable workflow captured from a real demonstration, not a generic written checklist.

This skill is part of the narrated Record & Replay plugin. Use
`narrated-record-replay:narrated-record-replay` for the coordinated capture
mechanics, privacy rules, transcript quality pipeline, alignment artifacts,
packet inspection, and replay voice preview. This skill owns the workflow
interpretation and the durable output.

## Fit

Use this when the demonstration will teach future agents:

- which UI actions are invariant;
- which values, files, accounts, filters, dates, or app surfaces vary;
- why the user chose one branch over another;
- what proof closes the workflow;
- what should be ignored as incidental motion.

Do not use this for a one-off support task where a short written answer is cheaper.

## Before Recording

Only direct invocation of this specialized skill authorizes its bounded capture.
Invoking the main narrated-record-replay skill or another companion skill does
not authorize this skill. If this skill was not directly invoked, do not start
capture or add microphone or postprocessing consent flags; use the main skill
for generalized capture or ask the user to invoke this skill directly.

After direct invocation: Do not ask pre-recording intake questions. Workflow
name, app surface, target artifact, success condition, max length, and
microphone consent are already bounded by the direct invocation or should be
learned from the recording.

1. Treat explicit invocation of this plugin skill as approval to run the bounded
   narrated workflow capture.
2. Use the default 30 minute recording limit unless the user already supplied a
   different duration.
3. Start through the coordinated `narrated-record-replay:narrated-record-replay`
   flow only. Do not treat sequential start as synchronized proof.
4. Follow the core `references/microphone-input-policy.md` before start. If
   auto input fails, retry a physical Mac default, AirPods, or MacBook
   microphone override before asking the user to repeat the workflow.
5. Tell the user only that recording is starting and that they should
   demonstrate the workflow while narrating decisions, defaults, branches, and
   success criteria.

If workflow name, output artifact, app surface, or success condition cannot be
inferred from the packet, ask targeted questions after recording and packet
review.

## During Recording

Listen for workflow-shaping statements:

- "always do this";
- "only do this when";
- "this value changes";
- "ignore this";
- "this is how I know it worked";
- "agents usually misunderstand this";
- "if this fails, recover by".

The user may pause, switch apps, or correct themself. Preserve self-corrections in the reviewed interpretation instead of smoothing away a decision change.

## After Recording

1. Stop Record & Replay and narration through the core plugin flow.
2. Build the packet. For a directly invoked narrated-workflow-capture run, add
   `--i-consent-to-openai-postprocessing` for the normal local-private
   transcript quality pipeline. Separate approval is still required before
   exporting raw audio, raw transcripts, secrets, broad private logs, or
   unredacted artifacts outside the local run.
3. Inspect:
   - `final-transcript-alignment.json`;
   - `temporal-context.json`;
   - `packet-inspection.json`;
   - `dogfood-receipt.json`;
   - Record & Replay metadata/events when provided.
4. Follow the core `references/transcript-video-alignment.md` before converting
   spoken guidance into workflow steps, variables, or proof requirements.
5. If alignment is low-confidence, use bounded action windows and record uncertainty. Do not pretend exact timing.

## Synthesis

Build the workflow from observed UI events plus cleaned aligned narration:

- Use Record & Replay events for screen/action claims.
- Use aligned final transcript for user intent, preferences, naming, and decision criteria only after checking the transcript/video alignment artifacts.
- Separate invariant steps from variables and optional branches.
- Convert narration into durable agent instructions only after removing raw private transcript detail.
- Include evidence requirements that future agents can actually verify.

The output should be concise enough for future agents to follow. Prefer a tight workflow artifact over a transcript summary.

Close any review/helper subagents after they finish so the session does not
leak limited agent slots.

## Output Contract

Return the workflow artifact path or inline artifact plus:

```text
Workflow artifact: <path or inline artifact>
Source run: <private session dir>
Record & Replay evidence: <metadata/events paths or missing>
Transcript source used: aligned final / batch raw / realtime raw / none
Claim ceiling: proven / requires operator review / blocked
Open risks: <short list>
```
