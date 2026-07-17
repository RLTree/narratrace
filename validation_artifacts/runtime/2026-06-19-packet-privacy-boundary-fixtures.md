---
last_edited: 2026-06-15
claim_ids: CLAIM-010, CLAIM-011, CLAIM-012
---

# Packet Privacy Boundary Fixture Receipt

## Purpose

Make generated packet inspection classify artifacts by privacy boundary. The
`inspect` command now writes `privacyBoundary` into `packet-inspection.json`,
separating raw local transcript inputs from generated review candidates and
marking the packet as not shareable without operator review.

## Changed Surface

- `src/inspect.rs`: adds `privacyBoundary.rawLocalOnly`,
  `privacyBoundary.distilledReviewCandidates`, `shareableStatus`, and
  `allowedToShareWithoutReview`.
- `tests/test_narrated_record_replay_inspect.py`: verifies raw transcript
  artifacts stay classified as `raw-local-private` and generated artifacts stay
  review candidates.
- `docs/design-docs/redaction-policy.md`: documents that packet inspection is
  the machine-readable privacy-boundary checklist for dogfood review.

## Focused Verification

Commands:

```sh
cargo fmt --manifest-path .codex/skills/narrated-record-replay/Cargo.toml -- --check
python3 -m pytest tests/test_narrated_record_replay_inspect.py
.codex/skills/narrated-record-replay/scripts/check
```

Results:

- `cargo fmt`: exit code `0`
- `pytest tests/test_narrated_record_replay_inspect.py`: `2 passed`
- `scripts/check`: exit code `0`; Cargo tests `0 passed; 0 failed`;
  pytest `26 passed`

## Claim Ceiling

This strengthens fixture-level privacy and review-boundary inspection for
`CLAIM-010`, `CLAIM-011`, and `CLAIM-012`. It still does not close those claims:
real non-toy packet usefulness review, raw-private leakage inspection on real
generated artifacts, and product-cohesion review remain owed.
