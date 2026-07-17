---
last_edited: 2026-06-21
---

# Non-Toy Packet Usefulness Density Gate

Scope: reviewed the refreshed 2026-06-21 non-toy coordinated dogfood packet for usefulness after the raw-transcript embedding fix. This note intentionally excludes raw transcript text.

Private run:

- Session dir: `/private/tmp/narrated-record-replay/1782011166-non-toy-narrated-record-replay-skill-refinement`
- Record & Replay session: `CC204231-AD46-4C09-8245-FE7C147B3282`

Issue found:

- The packet had strong evidence boundaries and privacy posture, but the narration content was too sparse for confident non-toy replay refinement.
- The refreshed temporal context had 23 Record & Replay events and 1 transcript segment.
- The transcript-derived context available for review was 18 words / 88 characters.
- That ratio is not enough for confident extraction of intent, variables, decision criteria, and success conditions for a non-toy replay skill.

Fix implemented:

- `skill-refinement-packet.md` now includes a `Narration Quality Summary` with transcript word count, transcript character count, density status, and usefulness warning.
- `packet-inspection.json` now includes machine-readable packet usefulness fields:
  - `transcriptWordCount`
  - `transcriptCharCount`
  - `recordReplayEventCount`
  - `narrationDensityStatus`
- Sparse non-toy narration adds a packet usefulness blocker: `narration is too sparse for confident non-toy replay refinement`.
- `review-contract.json` and `review-artifact.html` now expose narration density status and transcript counts.
- Sparse non-toy narration blocks the review contract status and adds a recovery action to run another coordinated dogfood with denser narration before reusing the packet.

Fresh artifact evidence:

```sh
jq '{status, signals: .packetUsefulnessReview.signals, blockers: .packetUsefulnessReview.blockers}' /private/tmp/narrated-record-replay/1782011166-non-toy-narrated-record-replay-skill-refinement/packet-inspection.json
jq '{status, reviewState: {narrationDensityStatus: .reviewState.narrationDensityStatus, transcriptWordCount: .reviewState.transcriptWordCount, transcriptCharCount: .reviewState.transcriptCharCount}, recoveryActions, productBlockers: .productCohesionReview.blockers}' /private/tmp/narrated-record-replay/1782011166-non-toy-narrated-record-replay-skill-refinement/review-contract.json
rg -n "Review state|Narration density status|Transcript words/chars|Run another coordinated dogfood" /private/tmp/narrated-record-replay/1782011166-non-toy-narrated-record-replay-skill-refinement/review-artifact.html
```

Observed results:

- `packet-inspection.json` status remained `requires-operator-review`.
- `packetUsefulnessReview.signals.narrationDensityStatus` is `too-sparse-for-non-toy-replay`.
- `packetUsefulnessReview.signals.transcriptWordCount` is `18`.
- `packetUsefulnessReview.signals.transcriptCharCount` is `88`.
- `packetUsefulnessReview.signals.recordReplayEventCount` is `23`.
- `review-contract.json` status is `blocked`.
- `review-artifact.html` shows `Review state: blocked`, `Narration density status: too-sparse-for-non-toy-replay`, and `Transcript words/chars: 18/88`.
- Recovery action is visible: run another coordinated dogfood with denser narration before reusing the packet for confident replay refinement.

Final refreshed artifact hashes:

- `packet-inspection.json`: `sha256:acd11f96e131071e54d31a3a3442b55b123bf933f10ed56c4af092909a2db4b5`
- `dogfood-receipt.json`: `sha256:18d1c892abd76bbbc69f9c0f15742eddd437cad59a6668dd04840d9f1be8b67d`
- `review-contract.json`: `sha256:61484d86c6969a9c2ef90f122eba0ddc9d74ea98d08ea29eac4da57a21ade816`
- `review-artifact.html`: `sha256:8f99bf419054d4a305c869119f6131f5cdb4dcd41b8adb04c1134d51ac923200`

Claim ceiling:

- This improves `CLAIM-011` evidence compiler usefulness by detecting that this real packet is not useful enough for confident replay refinement.
- It does not close `CLAIM-011`; it records a real non-toy usefulness review finding and a compiler/review-surface fix.
- The next dogfood must use coordinated Record & Replay plus microphone start, default Mac microphone selection, and denser narration that explains intent, variable inputs, decision criteria, and success conditions.
