---
last_edited: 2026-06-15
claim_ids: CLAIM-011
---

# Semantic Conflict Fixture Receipt

## Purpose

Strengthen fixture-level evidence-boundary diagnostics without claiming full packet usefulness. Transcript action claims now warn when they have nearby Record & Replay evidence but the nearby UI label obviously disagrees with the spoken action.

## Covered Behavior

- Narration saying `I clicked the save button.` with nearby Record & Replay labels `Cancel` and `Discard changes` produces a conflict warning.
- The warning reason is `nearby-ui-label-mismatch`.
- The warning keeps the existing `needs-ui-evidence` severity contract and adds operator-review language in the instruction.
- The evidence boundary report counts the warning.
- `timeline.rs` now delegates alignment diagnostics, Record & Replay event parsing, and conflict diagnostics to focused submodules.

## Verification

Command:

```sh
.codex/skills/narrated-record-replay/scripts/check
```

Result:

- Exit code: `0`
- Cargo tests: `0 passed; 0 failed`
- Pytest: `22 passed`

## Claim Ceiling

This supports conservative fixture-level evidence for `CLAIM-011`. It does not close `CLAIM-011`; a generated packet from a real non-toy workflow still needs inspection for relevance, completeness, privacy, and evidence-boundary quality.
