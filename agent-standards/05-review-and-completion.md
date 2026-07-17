# Review And Completion

## Review And Claim Ceiling

- Positive claims must map to claim ids in `GOAL_CONTRACT.md`.
- Missing proof lowers the claim ceiling; it does not become a note.
- Material review should falsify first: proof substitution, stale receipts,
  missing redaction, missing runtime proof, clock ambiguity, and
  human-attention overuse are blockers.
- Material review uses the current harness four-persona team: Contract and
  Claim Falsifier, Orchestration and Recovery Falsifier, Security
  Trust-Boundary Falsifier, and Product and Simplicity Falsifier.
- A single material review-team sign-off round is required for this lane: all
  four personas, fresh context, full scope, `gpt-5.5`, `high` reasoning, and
  same-round `SIGN_OFF` verdicts.
- Do not run extra model tiers or extra review rounds unless a reviewer finds a
  concrete blocker that requires repair and re-review.

## Completion Reports

For non-trivial work, report:

- Verification: exact command, artifact, receipt, runtime proof, or explicit
  gap.
- Security: privacy and secret risks reviewed, or `N/A`.
- Performance: latency, concurrency, file size, model-call, or runtime cost
  review, or `N/A`.
- Quality: tests, evals, runtime checks, review disposition, and residual gaps.
- Coverage: receipt path and claim ceiling, or explicit withheld coverage
  blocker. Do not present test pass counts as coverage.
- Claim ceiling: what is supported, what is unsupported, and what remains
  blocked.

## Plugin Packaging

- Source, installed plugin, and marketplace cache must be synced before claiming
  app-visible behavior changed.
- Bump plugin version when app-visible skill behavior or helper behavior
  changes.
- Run package hygiene on installed and cache roots.
