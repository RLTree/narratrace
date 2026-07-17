---
last_edited: 2026-06-15
---

# Temporal Context Contract

`temporal-context.json` is the machine-readable bridge between narration and Record & Replay artifacts.

## Schema

Current schema id: `narrated-record-replay.temporal-context.v1`

Required top-level fields:

- `schema`: contract version.
- `anchors`: source paths and available wall-clock anchors.
- `alignmentPolicy`: window, confidence, clock-assumption, and monotonic-clock policy.
- `capabilities`: what the packet may use the temporal context for.
- `transcriptSegments`: timestamped narration segments.
- `recordReplayEvents`: normalized Record & Replay events.
- `alignmentDiagnostics`: non-fatal timing and data-quality diagnostics.
- `alignments`: inferred transcript-to-UI-event windows.

## Anchors

`anchors.audioStartedAtUnixMs` is the narration wall-clock anchor. Transcript segment offsets are relative to that anchor.

`anchors.recordReplayStartedAtUnixMs` is parsed from Record & Replay metadata when available. It is used only for clock-skew diagnostics unless future live proof establishes stronger semantics.

`transcriptSegments[].monotonicOffsetMs` may exist when transcript events were captured by the live helper. It is a process-local elapsed millisecond offset from the narration capture loop, not an absolute monotonic timestamp that can be compared directly to Record & Replay. When present, `alignmentPolicy.monotonicClock.status` is `process-local-offsets-captured`.

## Production Timing Spine

The reliable design is a shared clock or a single muxed media stream:

- Best case: one capture process/container records screen video frames and
  microphone audio together, so audio and video presentation timestamps are
  naturally aligned. Record & Replay events then become supplemental UI
  evidence rather than the only timing bridge.
- If Record & Replay remains a separate official surface, the narration helper
  must record explicit `narration.sync.start` and `narration.sync.stop`
  sentinels with process-local monotonic timestamps. If the app can emit a
  harmless synthetic Record & Replay event for the same sentinels, use the pair
  to estimate offset and drift.
- Audio sample position is the strongest narration timing spine. Retained 24
  kHz PCM audio gives exact elapsed time by sample index. Runtime metadata
  should preserve `sampleStart`, `sampleEnd`, byte span, and monotonic capture
  offset per chunk without copying raw audio.
- Realtime transcription is rough word-time scaffolding, not final timing
  truth. Batch plus cleanup is final word authority, then cleaned words are
  mapped monotonically onto realtime/audio-token timing. Marker phrases are
  optional anchors only.

`process returned started`, websocket connect time, and transcript arrival time
are not alignment truth by themselves. They are diagnostics unless tied back to
audio sample position, shared monotonic timestamps, or muxed media timestamps.

## Confidence

Alignment confidence is based on absolute wall-clock delta between a transcript segment midpoint and a Record & Replay event timestamp:

- `high`: `<= 1000ms`
- `medium`: `<= 3000ms`
- `low`: `<= 6000ms`

Alignments outside `6000ms` are not emitted and are counted in diagnostics.

## Diagnostics

`alignmentDiagnostics` must report:

- missing audio anchors;
- missing Record & Replay metadata start anchors;
- malformed Record & Replay timestamps;
- events without timestamps;
- events outside the alignment window;
- Record & Replay start versus audio-start skew;
- repeated normalized transcript segment text.

Diagnostics are evidence boundaries, not automatic failures. A future validator can decide which diagnostics block a given claim.

## Claim Ceiling

This contract currently supports wall-clock timestamp-window alignment,
process-local monotonic transcript offsets, narration sync sentinels, and
audio chunk sample-span metadata. It does not prove cross-process monotonic
drift behavior unless Record & Replay emits comparable monotonic timestamps,
sync sentinels, or a muxed media surface. Post-facto alignment without common
timestamps is useful but not production-grade proof. `CLAIM-009` remains
blocked until live capture and monotonic drift evidence exist or the goal
contract explicitly accepts an unavailable-capability proof.
