---
last_edited: 2026-06-15
---

# Coordinated Start Live Proof Blocker

Date: 2026-06-19
Workspace: `/Users/terrynoblin/personal-monorepo`
Skill: `.codex/skills/narrated-record-replay`

## Purpose

Record the live dogfood correction that Record & Replay and microphone capture
must start as one coordinated operation. Sequentially starting Record & Replay
and then starting microphone capture is not valid evidence for `CLAIM-008`.

No raw audio, raw transcript text, or raw Record & Replay event content is
stored in this receipt.

## Live Attempts

Attempted sequential runs after explicit operator approval:

- Record & Replay session `F8978D4C-DED5-4B00-BC5B-BF28570472F6`: started
  first, then narration helper was started. The helper child exited before
  recording and left only `manifest.json` plus `status.json`.
- Record & Replay session `20F83F8B-B7CC-42F9-A436-2AD1F308149E`: repeated
  the sequential shape. The helper child exited before recording.
- Record & Replay session `F72E80D3-FAC7-47AC-A5D1-FD5AEB6C6111`: repeated
  after adding local-private child stdout/stderr files. The helper child exited
  before recording and stderr/stdout were empty.

Those attempts are negative evidence because the start shape itself was wrong:
the microphone should start automatically with the recording so both surfaces
share a start boundary.

Foreground fallback attempt:

- Record & Replay session `67B30CBB-3E2A-4EE8-BFEF-A22D8AE74E04`.
- Narration session:
  `/tmp/narrated-record-replay/1781909823-foreground-live-dogfood-capture-proof`.
- Record & Replay metadata path existed and was non-empty.
- Record & Replay events path existed and was non-empty.
- Narration helper status:

```json
{
  "state": "failed",
  "error": "Trying to work with closed connection",
  "maxSeconds": 30,
  "model": "gpt-realtime-whisper"
}
```

Generated structured artifacts from the failed foreground attempt:

- `evidence-boundary-report.json`: `recordReplayEvents=5`,
  `transcriptSegments=0`, `audioClockPresent=true`.
- `packet-inspection.json`: `status=requires-operator-review`; blockers
  include generated review candidate leak scan and real non-toy usefulness
  review.
- `dogfood-receipt.json`: `status=blocked`; blockers include
  `no transcript segments are present`.

## Code Changes

- `src/session/preflight.rs`: `liveDogfoodPlan` now requires coordinated start
  and sets `manualSequentialStartAllowedForLiveProof=false`.
- `src/session.rs`: the spawned internal `capture` command now receives the
  same microphone-consent and Record & Replay idle gates.
- `src/realtime.rs`: direct `capture` fails closed without
  `--i-consent-to-microphone-capture` and `--record-replay-status idle`.
- `src/receipt.rs`: `dogfood-receipt.json` blocks live claims unless the
  manifest proves Record & Replay and microphone capture started as one
  coordinated operation.
- `SKILL.md`: start instructions now forbid manual sequential start as live
  proof.

## Verification

```text
cargo fmt --manifest-path .codex/skills/narrated-record-replay/Cargo.toml -- --check
```

Result: exit 0.

```text
python3 -m pytest tests/test_narrated_record_replay_preflight.py tests/test_narrated_record_replay_receipt.py
```

Result: exit 0, `14 passed`.

## Claim Ceiling

This does not prove live narrated capture. It proves the previous sequential
dogfood path was invalid for live proof, records a real foreground capture
failure, and mechanically prevents uncoordinated receipts from closing
`CLAIM-008`.
