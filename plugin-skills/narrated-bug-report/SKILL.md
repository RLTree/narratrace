---
name: narrated-bug-report
description: Use when the user wants to reproduce a bug, confusing behavior, flaky workflow, broken UI state, or regression while narrating expected behavior, observed behavior, reproduction clues, and debugging hypotheses.
---

# Narrated Bug Report

Use this skill when the failure is best understood by watching the reproduction while the user explains what should have happened.

This skill is part of the narrated Record & Replay plugin. Use
`narrated-record-replay:narrated-record-replay` for coordinated capture,
local-private audio/transcript handling, alignment, inspection, and evidence
packet creation. This skill owns the bug report and debugging handoff.

## Fit

Use this for:

- UI bugs that need visual reproduction;
- timing-sensitive or flaky behavior;
- confusing states where the expected behavior is not obvious from logs;
- regressions where the user can show the broken path;
- bugs where user expectation matters as much as the visible failure.

Use normal debugging first for static compiler errors, obvious stack traces, or failures that do not benefit from visual reproduction.

## Before Recording

Only direct invocation of this specialized skill authorizes its bounded capture.
Invoking the main narrated-record-replay skill or another companion skill does
not authorize this skill. If this skill was not directly invoked, do not start
capture or add microphone or postprocessing consent flags; use the main skill
for generalized capture or ask the user to invoke this skill directly.

After direct invocation: Do not ask pre-recording intake questions. Expected
behavior, target surface, attempt count, max length, and microphone consent are
already bounded by the direct invocation or should be learned from the recording.

1. Treat explicit invocation of this plugin skill as approval to run the bounded
   narrated bug-report capture.
2. Use the default 30 minute recording limit unless the user already supplied a
   different duration.
3. Start through the coordinated `narrated-record-replay:narrated-record-replay`
   flow only. Do not treat sequential start as synchronized proof.
4. Follow the core `references/microphone-input-policy.md` before start. If
   auto input fails, retry a physical Mac default, AirPods, or MacBook
   microphone override before asking the user to reproduce the bug again.
5. Tell the user only that recording is starting and that they should reproduce
   the issue while narrating setup, expectation, surprise, failure moment, and
   hypotheses.

If expected behavior, failure target, or reproduction scope cannot be inferred
from the packet, ask targeted questions after recording and packet review.

## During Recording

Pay special attention to:

- exact reproduction steps;
- inputs, selected files, accounts, filters, dates, or toggles;
- the moment the user says the behavior diverges;
- visible error messages, empty states, loading stalls, flicker, or wrong data;
- user corrections such as "no, that's not it" or "this is the failure";
- silence during waiting periods, since the absence of action may matter.

If the user says a spoken value that intentionally differs from a typed value, preserve that distinction. Visible UI evidence and narration may both be true but describe different things.

## After Recording

1. Build and inspect the packet through the core plugin flow.
2. Follow the core `references/transcript-video-alignment.md` before tying
   spoken expectations, hypotheses, or "this is the failure" language to a
   visible UI moment.
3. Use Record & Replay events for reproduction steps and failure evidence.
4. Use aligned final transcript for expectation, surprise, and hypotheses only
   after checking `final-transcript-alignment.json`, `temporal-context.json`,
   and packet inspection.
5. If alignment is uncertain at the failure moment, report a bounded window
   instead of exact timing and lower the repro confidence.
6. If fixing in the same turn, read the relevant code and run focused validation. Keep the repro packet as the behavioral acceptance source.

## Bug Report Shape

Produce:

- one-sentence bug summary;
- environment/app surface;
- numbered reproduction steps;
- expected behavior;
- actual visible behavior;
- failure window;
- evidence artifacts;
- likely component or next investigation target;
- missing evidence and uncertainty.

Do not include raw private transcript text unless the exact wording is necessary and the user approved it.

Close any review/helper subagents after they finish so the session does not
leak limited agent slots.

## Output Contract

```text
Bug summary: <one sentence>
Repro confidence: high / medium / low
Failure window: <timestamp or bounded window>
Evidence run: <private session dir>
Next fix target: <module/file if known>
Claim ceiling: reproduced / suspected / blocked
```
