---
name: narrated-feature-implementation
description: Use when the user wants to show a UI, prototype, product flow, or code-adjacent surface while narrating the feature or behavior they want Codex to implement, including interaction details, edge states, acceptance criteria, and product intent.
---

# Narrated Feature Implementation

Use this skill when the user can specify the feature more accurately by showing the product and talking through the desired behavior.

This skill is part of the narrated Record & Replay plugin. Use
`narrated-record-replay:narrated-record-replay` for the capture, privacy,
transcription, alignment, inspection, and evidence packet. This skill owns the
feature interpretation and implementation handoff.

## Fit

Use this for implementation work where visible state matters:

- changing an existing UI flow;
- matching a prototype or current app surface;
- explaining interaction timing, copy, controls, disabled states, empty states, errors, or edge cases;
- showing what is wrong in the current product while describing the intended behavior.

Skip narrated capture for small code-only edits with an unambiguous written requirement.

## Before Recording

Only direct invocation of this specialized skill authorizes its bounded capture.
Invoking the main narrated-record-replay skill or another companion skill does
not authorize this skill. If this skill was not directly invoked, do not start
capture or add microphone or postprocessing consent flags; use the main skill
for generalized capture or ask the user to invoke this skill directly.

After direct invocation: Do not ask pre-recording intake questions. Target
surface, requested output, max length, and microphone consent are already
bounded by the direct invocation or should be learned from the recording.

1. Treat explicit invocation of this skill as approval to run the narrated feature implementation capture for this bounded run.
2. Use the default 30 minute recording limit unless the user already supplied a different duration.
3. Start through the coordinated `narrated-record-replay` flow only. Do not treat sequential start as synchronized proof.
4. Follow the core `references/microphone-input-policy.md` before start. If
   auto input fails, retry a physical Mac default, AirPods, or MacBook
   microphone override before asking the user to repeat the demo.
5. Tell the user only that recording is starting and that they should show the target surface while narrating the desired behavior, constraints, and acceptance criteria.

The agent should learn the target surface and requested output from the replay and transcript. If the packet cannot identify them, ask targeted questions after recording and packet review, not before.

## During Recording

Capture these feature-specific signals:

- current behavior versus desired behavior;
- controls that should be added, removed, renamed, enabled, or disabled;
- state transitions, loading behavior, and error handling;
- design constraints, density, accessibility expectations, and copy tone;
- implementation boundaries the user states explicitly;
- acceptance criteria and runtime proof the user expects.

If the user points to "this", "that", or "over here", use the aligned timing window and Record & Replay evidence to identify the visible target. If the target is ambiguous, keep the ambiguity in the brief.

## After Recording

1. Build and inspect the packet through the core plugin flow.
2. Follow the core `references/transcript-video-alignment.md` before treating
   spoken requirements as attached to a visible UI element or interaction.
3. Review `final-transcript-alignment.json` before trusting feature wording.
4. Review `temporal-context.json` to bind spoken requirements to visible UI
   windows, keeping timestamp-window links marked as inferred unless visible
   Record & Replay evidence proves the target.
5. Review `packet-inspection.json` for privacy and generated-artifact issues.
6. Read the live codebase before making code changes; narration is a specification source, not a substitute for code inspection.

## Planning Contract

After packet review and codebase reading, decide whether this is trivial or non-trivial.

Use a compact inline plan only when the feature is unambiguous, small, single-surface, and has an obvious validation path. For non-trivial work, create an ExecPlan file before editing.

Before writing a plan or ExecPlan, read:

1. `references/PLANS.md`
2. `templates/feature-implementation-execplan.md`

The ExecPlan must be based on the reviewed recording packet and live code reads. It should include:

- target surface and observed current behavior;
- desired behavior in user-facing terms;
- UI states, copy, controls, timing, and edge cases;
- transcript/video alignment evidence and low-confidence transcript/UI links;
- targeted user questions, if any;
- owned paths and forbidden paths;
- implementation units with repo-relative paths;
- exact acceptance checks and runtime proof;
- privacy and redaction boundaries;
- claim ceiling.

Ask targeted questions only after reviewing the recording, transcript alignment, and relevant code. Ask only questions that change implementation. Record the answer or explicit assumption in the compact plan or ExecPlan.

Once the compact plan or ExecPlan is finalized, bind the current agent to that implementation contract with the available goal tool. Use `set_goal()` where that is the exposed API, or the equivalent harness goal-binding tool. The goal must name the plan or ExecPlan, outcome, validation commands, and claim ceiling.

After goal binding, implement to completion against that contract unless a real blocker appears. Keep the plan current when implementation discovery changes scope, files, risks, or validation. Do not silently outrun the finalized plan.

When implementing, keep edits scoped to the demonstrated feature. Do not add speculative polish or broad refactors unless required to satisfy the captured acceptance criteria.
Close any review/helper subagents after they finish so the session does not
leak limited agent slots.

## Output Contract

Return either the implementation brief or patch closeout:

```text
Feature target: <surface>
Evidence run: <private session dir>
Plan contract: <inline compact plan or ExecPlan path>
Goal binding: <goal id/status or unavailable>
Implementation scope: <repo-relative files/modules or TBD>
Acceptance checks: <tests/runtime proof>
Transcript/action confidence: high / medium / low
Claim ceiling: ready to implement / implemented with proof / blocked
```
