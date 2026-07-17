---
last_edited: 2026-06-24
---

# Validation

The default gate for this skill directory is:

```sh
scripts/check
```

The gate intentionally avoids starting Record & Replay, recording audio, opening a microphone, or calling OpenAI. It proves setup integrity, helper compilation/tests, permission hygiene for local-private run artifacts, and non-live validation only.

## Gate Contents

Use the wrapper as the single local validation command:

```sh
scripts/check
```

From the installed plugin, run:

```sh
/Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/scripts/check
```

Do not replace it with hand-picked commands. The wrapper owns the current
Rust test set, including receipt and post-commit drain blockers.

In source mode, it also checks that the plugin retrofit setup files and routed directories exist, that `GOAL_CONTRACT.md` contains `CLAIM-001` through `CLAIM-013`, and that bundle JSON/JSONL files parse.

The wrapper runs `scripts/check-line-cap`, which fails if any active
hand-authored Rust source, skill contract, standards file, or Markdown doc
exceeds 250 lines. Generated lock files, machine JSON, validation artifacts,
build output, and worktree state are outside that source-size gate.

The wrapper also runs `scripts/check-skill-contracts`, which enforces
capture-first skill routing, microphone fallback guidance, transcript/video
alignment routing, no harness-process references in user-facing skills, and a
product-cohesion receipt.

The wrapper runs real Rust line coverage through `scripts/check-coverage`.
That script runs `cargo llvm-cov`, writes
`validation_artifacts/coverage/llvm-cov.json`, generates
`validation_artifacts/coverage/coverage-receipt.json`, validates the receipt,
and fails unless measured repo-owned Rust line coverage is exactly 100%.
Cargo tests remain test evidence, not coverage proof.

In source mode, the Rust bundle gate validates the current local bundle shape and recomputes artifact digests repeated in the completion manifest, latest amendment row, and backlog pointer:

- `LANE_REGISTRY.json` schema marker and lane array.
- `VERIFICATION_BACKLOG.json` schema marker and row array.
- `COMPLETION_MANIFEST.json` schema marker, canonical claim ids, evidence digests, required validator receipt paths, and backlog digest.
- `AMENDMENTS.jsonl` rows, schema markers, latest contract hash, and latest backlog update digests.

`RED_FIXTURES.json` is currently a local seed catalog and parse check, not the full plugin red-fixture corpus.

Bundle artifact paths in manifests and amendments must be repo-relative normal paths. Absolute paths, parent-directory traversal, symlinked artifact paths, and blocked validator receipts fail the Rust bundle gate. The local Rust bundle-validation receipt under `validation_artifacts/root-gate/` is not a final `ultragoal-audit` receipt.

The gate audits `/tmp/narrated-record-replay` modes without reading artifact contents: directories must be `700`, regular files must be `600`, and symlinks are blocked. Missing run root is allowed.

The Rust helper rejects unsafe local file authority before use: CLI roots,
session directories, and retained-audio paths must stay under
`/private/tmp/narrated-record-replay` unless
`--i-consent-to-custom-runtime-paths` is present; parent-directory components
and existing symlink components are rejected after macOS `/tmp` normalization;
private writes refuse final symlinks; external Record & Replay artifact reads
require regular non-symlink files. Custom ffmpeg audio filters require
`--i-consent-to-custom-audio-filter`.

After syncing the skill into an installed plugin or cache package, run:

```sh
scripts/check-package-hygiene /Users/terrynoblin/.codex/plugins/narrated-record-replay /Users/terrynoblin/.codex/plugins/cache/local-harness-plugins/narrated-record-replay/<version>
```

This gate rejects `.git`, `.codex-worktree`, local Cargo build output,
source-only proof ledgers, `validation_artifacts`, raw audio, raw transcript
artifacts, and dogfood receipts from packaged plugin copies. It is intentionally
separate from source-mode bundle validation because source proof ledgers are not
package runtime contracts.

Do not pass the repo-local source skill root to `check-package-hygiene` as if it
were a package root. The package roots are the installed plugin and versioned
marketplace cache copies.

`validate --json` requires `ffmpeg` to be installed. It reports whether `OPENAI_API_KEY` is present, but the default gate does not require the key and must not print it.

## Claim Mapping

| Check | Supports | Does not support |
| --- | --- | --- |
| Markdown/frontmatter tests | Skill discoverability and repo integrity. | Runtime behavior or UI capture. |
| Setup file presence check | Agent-first directory scaffold exists. | Quality or correctness of future feature claims. |
| Line-cap check | Active hand-authored Rust, skill, standards, and Markdown files stay under 250 lines. | Generated lock files, machine JSON, validation artifacts, or semantic code quality. |
| Skill-contract and product-cohesion linter | Capture-first routing, physical microphone fallback, transcript/video alignment guidance, and product-cohesion receipt are present. | Live user journey proof or operator-reviewed launch readiness. |
| Coverage gate | `cargo llvm-cov` produced a real line-coverage artifact and receipt. Current measured coverage is below 100%, so the gate blocks completion/readiness claims. | Product readiness, live runtime proof, or any claim that lower-than-100 coverage is acceptable. |
| Rust bundle and digest validation | Canonical setup contract, lane, backlog, manifest, amendment shape, strict artifact-path containment, passing local Rust validator receipt, and current repeated local artifact digests. | Full `ultragoal-audit` pass, complete JSON Schema parity, four-persona `gpt-5.5 high` sign-off, or live capability proof. |
| Private run permission audit | Local-private run artifacts under `/tmp/narrated-record-replay` are not group/world readable and do not contain symlinks. | Redaction correctness, transcript safety, or shareability. |
| Package hygiene gate | Installed/cache plugin bundles do not include source worktree metadata, build output, raw audio, raw transcripts, or dogfood receipts. | Repo-local worktree setup validity or live capability proof. |
| Safe path and forged-parent negative tests | Local private writes reject symlink/path traversal and dogfood receipts reject status-only parent-operation receipts. | Durable app/tool operation-id proof or full same-start proof. |
| Cargo format/test | Rust helper syntax and covered fixture behavior. | Live microphone, websocket, or Record & Replay integration. |
| Transcription quality fixture test | Default realtime delay, cleanup dictionary construction, same-pipeline packet consumption of pre-seeded batch/cleaned artifacts, final transcript alignment, and evidence/report surfacing. | Real retained audio contents, OpenAI batch transcription success, cleanup model availability, or video/event alignment quality. |
| Packet OpenAI postprocessing consent tests | `packet` fails closed before sending retained audio or transcript text to OpenAI unless `--i-consent-to-openai-postprocessing` is present, while local-only/fixture packet generation remains available. | User approval itself, successful OpenAI calls, or transcript quality. |
| `validate --json` | Local dependency detection and model name wiring. | Successful transcription or packet usefulness. |
| `delay-eval` and `delay-compare` fixture tests | Privacy-safe timing/alignment metric extraction and comparison for `high` versus `low` delay runs, including first audio chunk, first realtime delta, first completed realtime segment, final aligned utterance count, unresolved mismatches, and diagnostic scripted marker recall counts. | A completed A/B dogfood comparison with operator-reviewed transcript/action usefulness or any proof that `low` should replace `high` by default. |
| `preflight --record-replay-status idle` | Binds an externally observed Record & Replay idle status into the local preflight receipt. | Microphone consent, transcription, packet usefulness, or replay behavior. |
| `preflight` recommended command checks | Proves the emitted preparation command does not open the microphone and that same-operation capture remains withheld until fresh consent. | Consent itself, live capture, transcription, or packet usefulness. |
| `start` consent/status-gate negative tests | Proves live microphone capture fails closed without `--i-consent-to-microphone-capture` and `--record-replay-status idle`. | Successful live capture, transcription, or packet usefulness. |
| Packet fixture test | Basic temporal context generation from synthetic artifacts. | Real clock drift, real UI event fidelity, redaction completeness, or product usefulness. |

## Live Proof Required Later

Future positive feature claims require additional proof:

- Record & Replay availability check through the actual event-stream surface.
- Live narrated capture with microphone input and realtime transcription.
- Packet generation using actual Record & Replay metadata/events.
- Manual or reviewer inspection of generated artifacts for usefulness and privacy.
- Negative fixtures for redaction, conflicts, malformed events, missing anchors, and clock drift.
- Product-cohesion proof before claiming any review UI or operator control surface.
- A paired `delay-eval` comparison after separate `--delay high` and
  `--delay low` live runs before changing the default realtime delay.

## Security Scan Checkpoint (2026-07-17)

Codex Security scan rounds are patch-gated: land and verify the previous round's validated findings before starting another discovery round. Scan `755a3964-c486-4f7b-9459-d7d2a4b5a7ae` sealed 19 reportable findings; all 19 remediation patches now live in this source directory.

Fresh integrated evidence: `cargo test --locked -- --test-threads=1` passes 440/440, `cargo fmt --all -- --check` passes, `cargo check --locked` passes without warnings, and `scripts/check-line-cap` passes. Coverage includes exact audio-handle consent, v2 transcript/cache/alignment binding, conservative model-output validation, instruction/data separation, typed agent evidence, complete proof tuples, closed bundle semantics, and peer-acknowledged realtime commits.

The full `scripts/check` reaches the bundle proof gate and exits 1 with `trusted goal-service attestation unavailable`. This is intentional fail-closed behavior: caller-controlled environment and JSON observations no longer authenticate a goal or run. The earlier coverage-receipt provenance blocker remains separately unresolved, but this run does not reach that gate. Keep bundle completion, dogfood readiness, static-contract generation, coverage completion, package sync, and app-visible readiness withheld.

The next security claim requires a fresh deep scan over the remediated directory snapshot. A zero-finding scan is not implied by the passing source tests.

## Reporting Failures

If a command cannot run, record:

- Exact command.
- Exit code or blocker.
- Whether the blocker is local setup, sandbox, dependency, missing permission, missing runtime, or implementation failure.
- The supported claim ceiling after the gap.
