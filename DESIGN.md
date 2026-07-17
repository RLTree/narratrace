---
last_edited: 2026-06-15
---

# Design

Design work here is about making capture, review, and skill refinement coherent enough that agents do not misuse evidence.

## Current Surface

There is no complete visual review UI yet. Current user-facing surfaces are:

- `SKILL.md` instructions;
- CLI commands;
- generated Markdown and JSON packet artifacts;
- a static local `review-artifact.html` report plus `review-contract.json` for
  operator inspection.

## Design Requirements Later

- A future interactive review UI must show transcript context and Record & Replay evidence as separate surfaces.
- Alignment confidence must be visible.
- Conflict states must be understandable and recoverable.
- Redaction state must be visible before any artifact is shared.
- UI proof requires screenshots or app-visible evidence; CLI proof is insufficient.

## Current Design Gate

Keep instructions concise and route detail to the specialized docs. Do not
describe the static review artifact as a complete UI/control surface until
real UI runtime proof, product-cohesion proof, and recovery-state proof exist.
