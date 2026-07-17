---
last_edited: 2026-06-15
---

# Plans

Long-running work in this directory uses restartable ExecPlans.

## Active Plan

- `EXECPLAN.md` is the root plan for the narrated Record & Replay ultragoal.
- Future macro-lane plans live under `docs/exec-plans/active/`.
- Completed or abandoned lane plans move to `docs/exec-plans/completed/` only after their evidence and claim ceiling are recorded.

## Requirements

Each plan must include:

- purpose and observable outcome;
- current progress with dates and evidence;
- surprises and decisions;
- owned paths and forbidden paths;
- exact validation commands;
- proof surface for every positive claim;
- privacy and redaction requirements;
- recovery, teardown, and residual claim ceiling.

## Lane Extension

Macro-lanes must name:

- owner and role;
- workspace, branch, state roots, scratch roots, tool cache roots, artifact roots, and ports;
- consumed dependency claims and evidence digests;
- ready receipt path;
- live beneficial end-to-end proof requirement;
- teardown condition.

Do not use chat as the only plan. If the work would matter after an interruption, update the plan file.
