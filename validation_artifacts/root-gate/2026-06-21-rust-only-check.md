---
last_edited: 2026-06-15
---

# Rust-Only Local Gate

Command:

```sh
.codex/skills/narrated-record-replay/scripts/check
```

Working directory:

```text
/Users/terrynoblin/personal-monorepo
```

Observed at: 2026-06-21T03:30:00Z

Result: exit 0.

Coverage:

- Rust bundle validation receipt is checked in at `validation_artifacts/root-gate/2026-06-21-rust-bundle-validation.json`; the wrapper also writes a fresh temp receipt during each run.
- Private run permission audit completed.
- Shell syntax checks completed for `scripts/check`, `scripts/audit-private-run-permissions`, and `.codex/setup-worktree-env.sh`.
- `cargo fmt --check` completed.
- `cargo test` completed with 31 tests passed, including the 2026-06-21 scanner hardening tests.
- `validate --json` completed without printing secrets.
- Unknown-command negative check completed; typo command exits nonzero.
- Negative Rust coverage includes bundle absolute/parent/symlink proof-path rejection, status-only validator receipt rejection, generated-artifact symlink leak-scan blocking, and raw-local symlink no-hash behavior.

Claim ceiling:

This is local Rust gate proof only. It does not replace full plugin
`ultragoal-audit`, review-team sign-off, live registry exposure proof, live
non-toy dogfood, operator privacy/product review, or replay-engine execution
proof.
