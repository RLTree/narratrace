# Namespace And Progressive Disclosure

## File Size

- Active source, skill, standards, and hand-authored docs must stay at or below
  250 lines.
- Generated state and lock files can exceed 250 lines only when they are
  mechanically generated and not useful to split by hand. Do not use that
  exception for source code, skill contracts, standards, or runbooks.
- Before adding behavior to an oversized file, split the file first.
- The local gate must include a deterministic line-cap audit so this does not
  depend on memory or reviewer attention.

## Namespaces

- Paths must explain their domain responsibility before a file is opened.
- Avoid generic buckets such as `utils`, `helpers`, `misc`, or broad `common`
  for domain behavior.
- Repeated concepts should become named directories or typed records, not
  scattered prose.
- Split modules by product concepts: capture, audio input, realtime timing,
  post-stop transcription, cleanup, alignment, evidence, review, packaging.

## Skill Contracts

- `SKILL.md` files are entrypoints, not full manuals.
- Put shared operational detail in the core narrated skill or references, then
  have companion skills point to that contract.
- Companion skills must include critical startup constraints directly enough
  that a fresh agent does not ask intake questions or miss microphone consent.
- Do not reference harness-ultragoal docs from user-facing narrated plugin
  skills; harness docs are process guidance for building this plugin.

## Documentation Freshness

- Completion requires fresh affected docs. Update docs when work changes
  architecture, commands, standards, runtime behavior, product behavior, proof
  surfaces, lane state, operational procedure, validation receipts, backlog
  rows, or tech-debt records.
- Do not edit every doc every time. No affected repo-owned doc may be stale
  without owner, reason, required follow-up, and claim-ceiling impact.
- Generated docs must be regenerated through their generator unless their
  contract explicitly allows manual edits.
- Record owed updates in `VERIFICATION_BACKLOG.json`, the active ExecPlan,
  `agent-standards/enforcement.*`, or the final claim ceiling.
