# ExecPlans, Worktrees, And Orchestration

## ExecPlan Discipline

- `PLANS.md` is stable ExecPlan law, not project status, worker state, backlog,
  receipt state, or completion evidence.
- Long-running work uses `EXECPLAN.md` as the restartable plan and worklog.
- Progress entries name date, files touched, evidence gathered, surprises,
  blockers, and next action.
- TODO rows stay concrete enough for a future agent to execute without chat
  history.
- Stale plan claims must be corrected in the file, not patched over in chat.

## Worktree Environment

- Codex App-created worktrees must keep scratch, temp, Cargo target, and
  generated state under `.codex-worktree/`.
- Source `.codex-worktree/env.sh` before validation when working in a Codex App
  worktree.
- Do not hardcode a worktree path into environment setup.

## Orchestration

- Substantial work is organized around deliverables, proof, owners, and
  restartable state, not chat sessions.
- Parent sessions own dependency order, claim ceilings, package sync, and
  final proof reconciliation.
- Lane completion is not root completion; root claims need current root proof,
  clean package state, and review disposition.
