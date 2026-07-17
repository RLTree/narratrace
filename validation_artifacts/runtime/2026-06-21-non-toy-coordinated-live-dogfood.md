# 2026-06-21 Non-Toy Coordinated Live Dogfood

## Scope

Bounded live dogfood for the narrated Record & Replay ultragoal.

Goal: `non-toy narrated Record & Replay skill refinement dogfood`

This receipt records metadata, paths, counts, statuses, and hashes only. It does
not copy raw transcript text, raw audio, or raw Record & Replay event payloads.

## Coordinated Start

- Parent action: parallel `mcp__event_stream.event_stream_start` plus helper
  `capture` command with explicit microphone consent.
- Record & Replay session id: `CC204231-AD46-4C09-8245-FE7C147B3282`.
- Record & Replay started at: `2026-06-21T03:10:27Z`.
- Record & Replay ended at: `2026-06-21T03:12:34Z`.
- Helper session dir:
  `/private/tmp/narrated-record-replay/1782011166-non-toy-narrated-record-replay-skill-refinement`.
- Audio input selected by `--input auto`: `MacBook Pro Microphone`, `:1`,
  source `macos-default-input`.
- Parent-operation receipt status: `timestamp-proximity-verified`.
- Start delta: `3031 ms`.
- Max allowed start delta: `5000 ms`.
- Post-commit drain completed segments: `1`.
- Post-commit drain errors: `0`.

## Artifact Counts

- Transcript timeline rows: `20`.
- Transcript event rows: `25`.
- Record & Replay event rows: `23`.
- Evidence boundary aligned segments: `1`.
- Evidence boundary conflict warnings: `0`.
- Packet inspection status: `requires-operator-review`.
- Generated artifact leak scan status: `expected-local-references-only`.
- Dogfood receipt status: `requires-operator-review`.
- Review contract dogfood receipt status: `requires-operator-review`.
- Receipt command refreshed review contract and review artifact after writing
  `dogfood-receipt.json`.
- Replay voice preview status: `dry-run-not-spoken`.
- Replay voice preview cue count: `1`.

## Private Artifact Paths

- Record & Replay metadata:
  `/var/folders/6n/9_g0pjg51lg2s6hnlnf1q8hh0000gn/T/sky/event_stream/CC204231-AD46-4C09-8245-FE7C147B3282/session.json`
- Record & Replay events:
  `/var/folders/6n/9_g0pjg51lg2s6hnlnf1q8hh0000gn/T/sky/event_stream/CC204231-AD46-4C09-8245-FE7C147B3282/events.jsonl`
- Parent-operation receipt:
  `/private/tmp/narrated-record-replay/1782011166-non-toy-narrated-record-replay-skill-refinement/parent-operation-receipt.json`
- Packet inspection:
  `/private/tmp/narrated-record-replay/1782011166-non-toy-narrated-record-replay-skill-refinement/packet-inspection.json`
- Dogfood receipt:
  `/private/tmp/narrated-record-replay/1782011166-non-toy-narrated-record-replay-skill-refinement/dogfood-receipt.json`
- Review artifact:
  `/private/tmp/narrated-record-replay/1782011166-non-toy-narrated-record-replay-skill-refinement/review-artifact.html`
- Review contract:
  `/private/tmp/narrated-record-replay/1782011166-non-toy-narrated-record-replay-skill-refinement/review-contract.json`
- Replay voice execution plan:
  `/private/tmp/narrated-record-replay/1782011166-non-toy-narrated-record-replay-skill-refinement/replay-voice-execution-plan.json`

## Artifact Hashes

- `dogfood-receipt.json`:
  `sha256:eea13e4976086ffbe05b6cfface712a6ed23177f55013d3fdf10d0c1d0700eec`
- `review-contract.json`:
  `sha256:3d274d57ea608df43c538ec1c52285c4e855ee936370f1b72a3044c51dc3eeab`
- `review-artifact.html`:
  `sha256:a17f28e07c6010f08e9c0aac7072feffbbcb4fa9b78c0e46ec4407b02454f550`
- `parent-operation-receipt.json`:
  `sha256:95dfd7225eb0cb0727da89f88ead28e07ca62eed309a4b304aa128ae22b825bd`
- `packet-inspection.json`:
  `sha256:7749ac3b1f92cb4a4856e88e09695c734a50a97c03cfc9a74bccdeca4837c675`
- `evidence-boundary-report.json`:
  `sha256:73d809b637311be20130ec9c458f91920a75ac2143b5b01091827371af83ebcb`
- `replay-voice-execution-plan.json`:
  `sha256:be116ed99eb2938c930d0998c3651ffbc46ecbe5da8e6b57b207ef258e737d0f`

## Privacy Boundary

Dogfood receipt privacy fields:

- `rawAudioCopiedIntoReceipt=false`.
- `rawTextCopiedIntoReceipt=false`.
- `secretsCopiedIntoReceipt=false`.
- `allowedToShareWithoutReview=false`.

Packet inspection blockers:

- Real non-toy workflow packet usefulness inspection is still owed.
- Raw-private leakage inspection is still owed before sharing.

Dogfood receipt blocker:

- Operator review of generated artifacts is still required.

## Claim Ceiling

This is stronger live evidence for `CLAIM-008`, `CLAIM-009`, `CLAIM-011`, and
the dry-run portion of `CLAIM-013`, but it does not close those claims.

Unsupported after this run:

- Full live narrated capture claim closure, because operator review remains
  required before generated artifacts can be reused or shared.
- Complete audio/UI timeline correctness, because monotonic drift proof beyond
  timestamp proximity is still owed.
- Packet usefulness, because real non-toy operator review is still owed.
- Review UI completion, because product-cohesion/runtime review remains owed.
- Replay-time voice execution, because the preview did not speak audio or drive
  a replay engine.
