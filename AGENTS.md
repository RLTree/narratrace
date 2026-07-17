---
last_edited: 2026-06-15
---

# Narrated Record & Replay Agent Router

This file is the nearest operating contract for `.codex/skills/narrated-record-replay`.

Read in this order before non-trivial work in this skill directory:

1. `AGENT_STANDARDS.md`
2. `ARCHITECTURE.md`
3. `PLANS.md`
4. Specialized docs for the task: `SECURITY.md`, `RELIABILITY.md`, `PRODUCT_SENSE.md`, `DESIGN.md`, `FRONTEND.md`, or `QUALITY_SCORE.md`
5. `GOAL_CONTRACT.md`
6. `REFERENCES.md`
7. `EXECPLAN.md`
8. `VALIDATION.md`
9. `SKILL.md`
10. Relevant Rust source under `src/`

## Hard Rules

- Treat `AGENT_STANDARDS.md` as law for this skill directory.
- Preserve the distinction between Record & Replay UI evidence and microphone transcript context.
- Do not write raw audio, raw transcripts, raw private workflow details, secrets, credentials, or broad logs into durable skill files.
- Do not claim recorder/transcription integration, review UI, evidence compiler, replay voice controls, or production readiness from setup docs alone.
- Use `GOAL_CONTRACT.md` claim ids for completion claims and withheld claims.
- Use `EXECPLAN.md` for restartable task rows and progress, not chat-only state.
- Run `scripts/check` or document exactly why it could not run before a positive completion claim.
- Documentation freshness is part of completion. When work changes architecture,
  commands, standards, runtime behavior, product behavior, proof surfaces, lane
  state, operational procedure, generated-doc freshness, validation receipts,
  backlog rows, or tech-debt records, update affected repo-owned docs or record
  the stale-doc blocker with owner, reason, follow-up, and claim-ceiling impact.
- Keep the skill discoverable through `SKILL.md`; this directory must remain an actual Codex skill, not a loose runbook.
- Keep setup gaps in `docs/exec-plans/tech-debt-tracker.md`; do not hide missing proof in final prose.

## Current Scope

The initial setup scope is complete enough to continue implementation under the active ultragoal. Current work may improve the helper, receipts, review artifacts, and dogfood gates, but the full narrated recorder system remains unproven until every required claim in `GOAL_CONTRACT.md` has fresh named evidence.

Material review guidance is lane-local: require one four-person review-team sign-off round using `gpt-5.5` with high reasoning, unless the user explicitly amends the policy again. Do not require legacy staged review gates, extra model tiers, or stale installed-cache review ladders.
