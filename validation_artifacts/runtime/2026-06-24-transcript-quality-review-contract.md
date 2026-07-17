# Transcript Quality Review Contract Surface

Date: 2026-06-24

Scope: Make disabled or missing post-stop transcript-quality layers visible in
the operator review contract and static review artifact, then block review
reuse while those layers are incomplete.

Private runtime session inspected:

- `/private/tmp/narrated-record-replay/1782264817-narrated-record-and-replay-capture`

The session is not live proof for the full narrated recorder. Its batch
transcription, cleanup, and final cleaned alignment receipts are disabled. This
artifact records only receipt status and reasons, not raw transcript text or
audio.

Evidence:

- `review-contract.json` now includes
  `reviewState.transcriptQualityPipeline`.
- `review-artifact.html` now includes a `Transcript quality receipts` line.
- Review status and product cohesion now treat disabled or missing transcript
  quality receipts as blockers.
- `inspect` generated `packet-inspection.json` and refreshed the review
  contract without API calls or audio capture.
- The regenerated local contract reported:
  - batch transcription receipt: `disabled`, reason `disabled-by-config`
  - cleanup receipt: `disabled`, reason `disabled-by-config`
  - final alignment receipt: `disabled`, reason `missing-cleaned-transcript`
  - packet inspection status: `requires-operator-review`
  - generated artifact leak scan status: `expected-local-references-only`
  - narration density status: `too-sparse-for-non-toy-replay`
  - product blocker: `transcript quality pipeline is incomplete or disabled`
  - recovery action: regenerate with post-stop batch transcription, cleanup,
    and final alignment enabled before trusting final words

Verification:

```sh
cargo fmt --manifest-path /Users/terrynoblin/personal-monorepo/.codex/skills/narrated-record-replay/Cargo.toml -- --check
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cargo-target cargo run --manifest-path /Users/terrynoblin/personal-monorepo/.codex/skills/narrated-record-replay/Cargo.toml -- review --session-dir /private/tmp/narrated-record-replay/1782264817-narrated-record-and-replay-capture
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cargo-target cargo run --manifest-path /Users/terrynoblin/personal-monorepo/.codex/skills/narrated-record-replay/Cargo.toml -- inspect --session-dir /private/tmp/narrated-record-replay/1782264817-narrated-record-and-replay-capture
jq '.reviewState.transcriptQualityPipeline' /private/tmp/narrated-record-replay/1782264817-narrated-record-and-replay-capture/review-contract.json
grep -n "Transcript quality receipts" /private/tmp/narrated-record-replay/1782264817-narrated-record-and-replay-capture/review-artifact.html
bash /Users/terrynoblin/personal-monorepo/.codex/skills/narrated-record-replay/scripts/check
```

Result:

- Focused review tests passed.
- Source gate passed with 98 Rust tests.
- Claim ceiling remains unchanged: this improves review legibility for blocked
  transcript-quality runs, but does not prove live audio capture, batch
  transcription, cleanup, final alignment quality, or video/event alignment.
