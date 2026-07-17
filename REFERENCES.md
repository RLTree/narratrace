---
last_edited: 2026-06-15
---

# References

Use these sources before planning substantial changes to narrated Record & Replay.

## Local Skill Sources

- `.codex/skills/narrated-record-replay/SKILL.md`: discoverable skill entrypoint and user-facing run contract.
- `.codex/skills/narrated-record-replay/Cargo.toml`: Rust helper package and dependencies.
- `.codex/skills/narrated-record-replay/src/`: current helper implementation for CLI parsing, session state, realtime capture, timeline alignment, and packet generation.
- `.codex/skills/narrated-record-replay/scripts/check`: current Rust-only validation gate for this skill.
- Historical root Python tests may describe earlier coverage, but they are not the current closure surface for this skill.

## Repo-Local Process Sources

- `AGENTS.md`: root personal-monorepo instructions.
- `README.md`: repo shape and skill discovery conventions.

## Harness Ultragoal Plugin Sources

Use the installed Harness Ultragoal plugin as the current process authority for
this ultragoal setup. The older proposal repo is historical context only unless
Tree explicitly asks to inspect it.

- `/Users/terrynoblin/.codex/plugins/harness-ultragoal/skills/harness-engineering/SKILL.md`: umbrella routing for agent-first repo setup.
- `/Users/terrynoblin/.codex/plugins/harness-ultragoal/skills/agent-first-repo-retrofit/SKILL.md`: existing-repo retrofit sequence and acceptance bar.
- `/Users/terrynoblin/.codex/plugins/harness-ultragoal/skills/proof-gate/SKILL.md`: proof-surface and claim-ceiling refusal rules.
- `/Users/terrynoblin/.codex/plugins/harness-ultragoal/skills/standards-gardener/SKILL.md`: recurring standards-drift cleanup.
- `/Users/terrynoblin/.codex/plugins/harness-ultragoal/templates/AGENT_STANDARDS.md`: current standards router template.
- `/Users/terrynoblin/.codex/plugins/harness-ultragoal/templates/agent-standards/`: current routed standards modules and enforcement templates.
- `/Users/terrynoblin/.codex/plugins/harness-ultragoal/templates/COVERAGE_RECEIPT.json`: coverage receipt authoring template.
- `/Users/terrynoblin/.codex/plugins/harness-ultragoal/schemas/coverage-receipt.schema.json`: coverage receipt schema.
- `/Users/terrynoblin/.codex/plugins/harness-ultragoal/docs/agent-first-repo-shape.md`: reference shape for agent-first routing, standards, plans, proof roots, and check wrappers.

## Non-Authority For This Work

- `.codex/skills/ultragoal/SKILL.md`: useful for ordinary local durable-goal work, but not the correct authority for this narrated Record & Replay ultragoal setup.

## Memory And Wiki Result

The explicit OpenClaw preflight for this setup found no source-backed wiki result for the exact narrated Record & Replay query. LanceDB recall returned harness-routing guidance: route reusable procedures to skills, durable knowledge to wiki or memory, irreversible actions to scoped gates, and avoid cargo-culting harness machinery without task-shaped need.

Treat that recall as context only. Verify all implementation and process claims against the files above.
