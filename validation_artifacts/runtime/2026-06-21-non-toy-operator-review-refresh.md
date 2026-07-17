---
last_edited: 2026-06-21
---

# Non-Toy Operator Review Refresh

Scope: refreshed the 2026-06-21 non-toy coordinated dogfood artifacts with the installed Rust plugin after reviewing the generated packet, receipt, review contract, and Record & Replay session metadata. This note intentionally excludes raw transcript text.

Private run:

- Session dir: `/private/tmp/narrated-record-replay/1782011166-non-toy-narrated-record-replay-skill-refinement`
- Record & Replay session: `CC204231-AD46-4C09-8245-FE7C147B3282`
- Record & Replay metadata: `/var/folders/6n/9_g0pjg51lg2s6hnlnf1q8hh0000gn/T/sky/event_stream/CC204231-AD46-4C09-8245-FE7C147B3282/session.json`
- Record & Replay events: `/var/folders/6n/9_g0pjg51lg2s6hnlnf1q8hh0000gn/T/sky/event_stream/CC204231-AD46-4C09-8245-FE7C147B3282/events.jsonl`

Commands run:

```sh
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-plugin-cargo-target cargo run --quiet --manifest-path /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/Cargo.toml -- packet --session-dir /private/tmp/narrated-record-replay/1782011166-non-toy-narrated-record-replay-skill-refinement --recording-metadata /var/folders/6n/9_g0pjg51lg2s6hnlnf1q8hh0000gn/T/sky/event_stream/CC204231-AD46-4C09-8245-FE7C147B3282/session.json --recording-events /var/folders/6n/9_g0pjg51lg2s6hnlnf1q8hh0000gn/T/sky/event_stream/CC204231-AD46-4C09-8245-FE7C147B3282/events.jsonl
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-plugin-cargo-target cargo run --quiet --manifest-path /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/Cargo.toml -- parent-operation-receipt --session-dir /private/tmp/narrated-record-replay/1782011166-non-toy-narrated-record-replay-skill-refinement --recording-metadata /var/folders/6n/9_g0pjg51lg2s6hnlnf1q8hh0000gn/T/sky/event_stream/CC204231-AD46-4C09-8245-FE7C147B3282/session.json --recording-events /var/folders/6n/9_g0pjg51lg2s6hnlnf1q8hh0000gn/T/sky/event_stream/CC204231-AD46-4C09-8245-FE7C147B3282/events.jsonl
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-plugin-cargo-target cargo run --quiet --manifest-path /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/Cargo.toml -- inspect --session-dir /private/tmp/narrated-record-replay/1782011166-non-toy-narrated-record-replay-skill-refinement
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-plugin-cargo-target cargo run --quiet --manifest-path /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/Cargo.toml -- review --session-dir /private/tmp/narrated-record-replay/1782011166-non-toy-narrated-record-replay-skill-refinement
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-plugin-cargo-target cargo run --quiet --manifest-path /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/Cargo.toml -- receipt --session-dir /private/tmp/narrated-record-replay/1782011166-non-toy-narrated-record-replay-skill-refinement --recording-metadata /var/folders/6n/9_g0pjg51lg2s6hnlnf1q8hh0000gn/T/sky/event_stream/CC204231-AD46-4C09-8245-FE7C147B3282/session.json --recording-events /var/folders/6n/9_g0pjg51lg2s6hnlnf1q8hh0000gn/T/sky/event_stream/CC204231-AD46-4C09-8245-FE7C147B3282/events.jsonl
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-plugin-cargo-target cargo run --quiet --manifest-path /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/Cargo.toml -- inspect --session-dir /private/tmp/narrated-record-replay/1782011166-non-toy-narrated-record-replay-skill-refinement
```

Review result:

- The recording used `MacBook Pro Microphone` as the selected `auto` input, with `ffmpegInput=:1` and source `macos-default-input`.
- Parent operation receipt remained `timestamp-proximity-verified`; start delta was 3031 ms.
- Post-commit drain completed with 1 completed segment, 5 messages, 0 errors, and no stop timeout artifact.
- Record & Replay evidence remained present: 23 events, 1 aligned transcript segment, 0 conflict warnings.
- Replay voice remains a dry-run plan only: 1 cue, `dry-run-not-spoken`, no replay engine receipt.
- Generated review candidates no longer embed the known captured transcript phrase. Verification command used `rg -l` over `skill-refinement-packet.md`, `thought-process.md`, `review-artifact.html`, `packet-inspection.json`, and `dogfood-receipt.json`; result was no matches.
- `packet-inspection.json` reports `hasTranscriptReviewBoundarySection=true` and `rawTranscriptEmbeddingAvoided=true`.
- Generated artifact leak scan status is `expected-local-references-only`; findings are expected local paths and opaque artifact ids, not blocking content.
- Raw transcript artifacts remain local-private with counts and fingerprints only in receipts.

Final refreshed artifact hashes:

- `skill-refinement-packet.md`: `sha256:5b814e71a1e5a4d3c9995380bf2ab5f1e750e1a2466e4ac3cdf643e616d30aba`
- `thought-process.md`: `sha256:db5db4d638e5042e0bdfd3d0b2092db27d3eb0a1451c2ee27367970d0bb234f1`
- `packet-inspection.json`: `sha256:de1e472200955ff7d4d5c58a0620db03dcede16dbc593ace5c1d9f0c999a58cc`
- `dogfood-receipt.json`: `sha256:6829837153a0356c1604287d06ddbfa780fc04b1b75082ff328bced4b8c24a04`
- `review-contract.json`: `sha256:1844daa1b55eada81d5b9437e30a71089b6a03d47336eda38004d7e77837fd87`
- `review-artifact.html`: `sha256:d588c0695685e008323afe3833671087e9517e2fdcc44c34f61dd92f3febbd42`
- `replay-voice-execution-plan.json`: `sha256:be116ed99eb2938c930d0998c3651ffbc46ecbe5da8e6b57b207ef258e737d0f`

Validation:

```sh
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-plugin-cargo-target /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/scripts/check
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cargo-target-check /Users/terrynoblin/personal-monorepo/.codex/skills/narrated-record-replay/scripts/check
find /Users/terrynoblin/personal-monorepo/.codex/skills/narrated-record-replay /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay -maxdepth 1 \( -name target -o -name cargo-target \) -print
```

Results:

- Installed plugin `scripts/check`: exit 0; 41 Rust tests passed.
- Source skill `scripts/check`: exit 0; 41 Rust tests passed.
- Skill-local build-output check: no `target` or `cargo-target` directories printed.

Claim ceiling:

- This closes the observed raw-transcript embedding bug in generated packet/review candidates for this run.
- This does not close browser/runtime review UI proof, monotonic drift proof, live replay voice execution, or final review-team sign-off.
- `requires-operator-review` remains the correct status until product-cohesion review, real packet usefulness review, and the remaining live proof obligations are satisfied.
