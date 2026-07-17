# Dense Narration Preflight Guidance

## Context

Review of real narrated Record & Replay dogfood sessions showed that the recorder can
produce usable packet/review artifacts while still failing the non-toy replay
usefulness bar when the transcript is empty or too sparse.

Relevant reviewed runs:

- `/private/tmp/narrated-record-replay/1782016380-review-the-gibson-nebuilder-lic-markdown-report`
  - Produced distilled workflow value, but the helper stop path was not clean and the
    receipt was blocked.
- `/private/tmp/narrated-record-replay/1782019206-patched-empty-completion-and-default-mic-smoke`
  - Stopped cleanly with default microphone selection, but packet inspection reported
    zero transcript segments.
- `/private/tmp/narrated-record-replay/1782011166-non-toy-narrated-record-replay-skill-refinement`
  - Had 23 Record & Replay events but only 18 transcript words, so the review contract
    correctly blocked confident non-toy replay reuse.

## Change

The Rust CLI now exposes machine-readable narration quality targets before capture
starts:

- `preflight --json` includes `narrationQualityTargets`.
- `liveDogfoodPlan` includes the same `narrationQualityTargets`.
- `prepare-coordinated-session` stdout includes `narrationQualityTargets`.
- Prepared `manifest.json` includes `narrationQualityTargets`.

The target is intentionally guidance, not proof. Packet inspection and the review
contract remain the post-capture authority.

## Verification

Source skill check:

```text
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cargo-target-check /Users/terrynoblin/personal-monorepo/.codex/skills/narrated-record-replay/scripts/check
exit 0
```

Targeted regression:

```text
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cargo-target-check cargo test --manifest-path /Users/terrynoblin/personal-monorepo/.codex/skills/narrated-record-replay/Cargo.toml narration_quality_targets_are_machine_readable -- --nocapture
running 1 test
test session::tests::narration_quality_targets_are_machine_readable ... ok
test result: ok. 1 passed; 0 failed; 42 filtered out
```

Installed plugin check:

```text
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-plugin-cargo-target /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/scripts/check
running 43 tests
test session::tests::narration_quality_targets_are_machine_readable ... ok
test result: ok. 43 passed; 0 failed
```

Installed plugin preflight smoke:

```text
cargo run --quiet --manifest-path /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/Cargo.toml -- preflight --json --goal 'dense narration guidance smoke' --root /private/tmp/narrated-record-replay-guidance-smoke --record-replay-status idle --max-seconds 60 | jq '{narrationQualityTargets, liveDogfoodTargets: .liveDogfoodPlan.narrationQualityTargets, operatorActionsRequired}'
```

Observed target fields:

```json
{
  "minimumTranscriptWordsForNonToyReplay": 30,
  "recommendedTranscriptWordsForNonToyReplay": 80,
  "recommendedTranscriptSegmentsForNonToyReplay": 3,
  "densityGate": "packet-inspection.narrationDensityStatus must not be too-sparse-for-non-toy-replay before confident non-toy replay reuse"
}
```

Installed plugin prepare/manifest smoke:

```text
cargo run --quiet --manifest-path /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/Cargo.toml -- prepare-coordinated-session --goal 'dense narration guidance smoke' --root /private/tmp/narrated-record-replay-guidance-smoke --max-seconds 60
```

Prepared manifest verified:

```text
/private/tmp/narrated-record-replay-guidance-smoke/1782022538-dense-narration-guidance-smoke/manifest.json
```

`manifest.json` includes `narrationQualityTargets` with the same minimum,
recommended, checklist, density gate, and claim ceiling fields.

## Claim Ceiling

This closes the setup discoverability gap for narration density targets. It does not
close live replay usefulness, monotonic drift proof, replay-time voice execution, or
the speech stop-command issue.
