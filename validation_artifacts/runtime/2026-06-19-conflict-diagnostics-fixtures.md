---
last_edited: 2026-06-15
---

# Conflict Diagnostics Fixture Receipt

Date: 2026-06-19

Scope: initial executable evidence-boundary fixture for `CLAIM-011`.

## Command

```sh
.codex/skills/narrated-record-replay/scripts/check
```

Result: exit 0.

Pytest reported 12 passed.

## Covered Behavior

- `temporal-context.json` includes a `conflictPolicy` block.
- Transcript segments that mention simple action verbs with no nearby Record & Replay event are emitted in `conflictDiagnostics.warnings`.
- Transcript action claims with nearby Record & Replay support are not warned by this heuristic.
- Missing audio anchors cause action claims to be marked as needing UI evidence.
- `skill-refinement-packet.md` reports the count of transcript action claims needing UI evidence.

## Claim Ceiling

Supported:

- The evidence compiler now emits machine-readable warnings for simple transcript action claims that lack nearby UI evidence.
- Generated packets surface the warning count for future skill refinement.

Unsupported:

- Full semantic conflict detection.
- Real-workflow usefulness inspection.
- Product-quality review of generated refinement packets.
- Any claim that the evidence compiler is complete for serious workflows.
