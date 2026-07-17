---
last_edited: 2026-06-24
---

# Agent Standards

This file is the always-loaded standards router for narrated Record & Replay
work. Treat it and every routed `agent-standards/` module as law. Load the
matching module before editing code, skills, docs, validation, packaging, or
review surfaces.

## Routing

- Namespaces, progressive disclosure, and file-size limits:
  `agent-standards/01-namespace-and-progressive-disclosure.md`
- Documentation freshness and generated-doc handling:
  `agent-standards/01-namespace-and-progressive-disclosure.md`
- Evidence boundaries, validation, parsing, and mechanical enforcement:
  `agent-standards/02-boundaries-validation-and-enforcement.md`
- ExecPlans, worktrees, and orchestration:
  `agent-standards/03-execplans-worktrees-and-orchestration.md`
- Security, reliability, runtime proof, and product cohesion:
  `agent-standards/04-security-reliability-and-product-cohesion.md`
- Review, claim ceilings, completion reports, and plugin packaging:
  `agent-standards/05-review-and-completion.md`
- Recurring friction, standards gardening, and self-improvement:
  `agent-standards/06-standards-gardening.md`

## Always-On Law

- Live files, current commands, runtime receipts, and current artifacts outrank
  chat, memory, summaries, and prior handoffs.
- Raw audio, raw transcripts, secrets, private workflow details, and broad logs
  are local-private unless Tree explicitly approves a distilled, scoped export.
- The validation gate is `scripts/check`; do not weaken or narrow it to hide a
  failure.
- Coverage proof is required for coverage, readiness, release, or production
  claims. Test pass counts and fixture checks are not coverage proof.
- `SKILL.md` files are user-facing contracts. Keep them compact and route
  details through references instead of creating mega skill files.
- Missing proof lowers the claim ceiling. It does not become a footnote.
