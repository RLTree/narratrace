---
name: narrated-review-debrief
description: Use when the user reviews a report, research output, document, dashboard, design, generated artifact, code result, or agent-produced work while narrating acceptance criteria, concerns, edits, and decision reasoning.
---

# Narrated Review Debrief

Use this skill when the user's review reasoning is the durable artifact: what passed, what failed, what needs revision, and which criteria future agents should preserve.

This skill is part of the narrated Record & Replay plugin. Use
`narrated-record-replay:narrated-record-replay` for coordinated capture,
privacy, transcript quality, alignment, inspection, and packet creation. This
skill owns the review debrief and follow-up extraction.

## Fit

Use this for reviews of:

- research reports and generated analysis;
- dashboards, documents, slide decks, or designs;
- code results or implementation demos;
- agent-produced artifacts where acceptance criteria should feed future work;
- workflow-engine outputs where the user's reasoning is hard to reconstruct from clicks alone.

Skip narrated capture for quick approval checks with no reusable reasoning.

## Before Recording

Only direct invocation of this specialized skill authorizes its bounded capture.
Invoking the main narrated-record-replay skill or another companion skill does
not authorize this skill. If this skill was not directly invoked, do not start
capture or add microphone or postprocessing consent flags; use the main skill
for generalized capture or ask the user to invoke this skill directly.

After direct invocation: Do not ask pre-recording intake questions. Review
target, desired output, max length, and microphone consent are already bounded
by the direct invocation or should be learned from the recording.

1. Treat explicit invocation of this plugin skill as approval to run the bounded
   narrated review-debrief capture.
2. Use the default 30 minute recording limit unless the user already supplied a
   different duration.
3. Start through the coordinated `narrated-record-replay:narrated-record-replay`
   flow only. Do not treat sequential start as synchronized proof.
4. Follow the core `references/microphone-input-policy.md` before start. If
   auto input fails, retry a physical Mac default, AirPods, or MacBook
   microphone override before asking the user to repeat the review.
5. Tell the user only that recording is starting and that they should review the
   artifact while narrating verdicts, acceptance criteria, dealbreakers,
   revisions, and points future agents should preserve.

If review target, requested output, or acceptance criteria cannot be inferred
from the packet, ask targeted questions after recording and packet review.

## During Recording

Capture review-specific signals:

- section or UI area being inspected;
- explicit verdicts;
- concerns, doubts, and corrections;
- acceptance criteria and examples;
- comments about source quality, missing evidence, formatting, accessibility, or product fit;
- follow-up tasks and priority;
- reusable rubric changes.

If the user scrolls, hovers, selects text, or pauses silently, use the aligned event window to identify what was visible. Do not infer content that was not visible or read from an artifact.

## After Recording

1. Build and inspect the packet through the core plugin flow.
2. Follow the core `references/transcript-video-alignment.md` before tying
   review comments to a visible section, selected text, scroll position, chart,
   code result, or artifact region.
3. Use aligned final transcript for the user's reasoning only after checking
   `final-transcript-alignment.json`, `temporal-context.json`, and packet
   inspection.
4. Use Record & Replay events, screenshots, or artifact reads for claims about visible content.
5. If the user supplies corrections after recording, record those as post-run user corrections, not transcription output.
6. Distinguish subjective user preference from objective artifact defects.

## Debrief Shape

Produce:

- reviewed artifact and scope;
- verdict if stated;
- findings tied to visible sections or action windows;
- user acceptance criteria;
- transcript/video alignment confidence for important findings;
- follow-up tasks with priority;
- reusable rubric or skill updates only when justified;
- uncertainty and evidence gaps.

Do not store raw private review transcripts in repo files, memory, or shared docs. Distill the reasoning.

Close any review/helper subagents after they finish so the session does not
leak limited agent slots.

## Output Contract

```text
Review target: <artifact/surface>
Verdict: pass / revise / block / no verdict
Top follow-ups: <short list>
Reusable criteria: <short list or none>
Evidence run: <private session dir>
Claim ceiling: operator-reviewed / requires operator review / blocked
```
