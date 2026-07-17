---
last_edited: YYYY-MM-DD
---

# <Feature Name> ExecPlan

## Purpose

<What user-visible behavior will change, and how success will be observed.>

## Source Evidence

- Narrated session: `<private session dir>`
- Record & Replay metadata: `<path or missing>`
- Record & Replay events: `<path or missing>`
- Transcript source used: `aligned final | batch raw | realtime raw | none`
- Final transcript alignment: `<final-transcript-alignment.json path or missing>`
- Temporal context: `<temporal-context.json path or missing>`
- Transcript/video confidence: `high | medium | low | unresolved`
- Unresolved transcript/video mismatches: `<count and short summary>`
- Deictic bindings: `<this/that/over here bindings with visible evidence or unresolved>`
- Packet inspection: `<path or missing>`

## Current Behavior

<Observed behavior from screen evidence and code reads. Cite uncertainty explicitly.>

## Desired Behavior

<Desired behavior from cleaned aligned narration and user answers.>

## Non-Goals

- <Explicitly out-of-scope behavior.>

## Decisions And Assumptions

- <Decision or assumption, with rationale and evidence source.>

## Targeted Questions

- [ ] <Question that would change implementation. Remove if answered or not needed.>

## Owned Paths

- `<repo-relative path>`: <why this path is in scope>

## Forbidden Paths

- `<repo-relative path>`: <why this path must not be touched>

## Implementation Units

1. `<unit name>`
   - Paths: `<repo-relative paths>`
   - Behavior: <what changes>
   - Tests: `<repo-relative test paths or runtime checks>`
   - Proof: <how this unit is verified>

## Validation

```sh
<exact command>
```

## Privacy And Redaction

- Raw audio remains local-private.
- Raw realtime, batch, and cleaned transcripts are not copied into repo files by default.
- Durable files include only distilled requirements and evidence references.

## Recovery And Rollback

<How to recover if implementation fails or validation regresses.>

## Claim Ceiling

<What can be claimed after validation, and what remains unproven.>
