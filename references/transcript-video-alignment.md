# Transcript-To-Video Alignment

Do not assume the transcript is automatically aligned with the video just
because `packet` produced transcript artifacts. The helper creates candidate
alignment artifacts; the agent must verify them before using narration to
identify what was visible on screen.

For every post-recording task:

1. Inspect `final-transcript-alignment.json` first. Use cleaned aligned text as
   the preferred word source only when the file exists and unresolved mismatches
   are low enough for the task.
2. Inspect `temporal-context.json`. Treat transcript-to-Record-and-Replay
   windows as inferred unless a Record & Replay event, screenshot, artifact
   read, or visible UI state directly proves the claim.
3. Inspect timing proof when present: `parent-operation-receipt.json`,
   `capture-clock.json`, `audio-chunks.jsonl`, `narration.sync.jsonl`, Record &
   Replay metadata, and Record & Replay events. Use audio sample/chunk timing
   and explicit sync sentinels as stronger evidence than transcript arrival
   time, websocket timing, or process-start time.
4. For deictic phrases such as "this", "that", "over here", "right there", or
   "the thing I just clicked", bind the phrase to the nearest verified
   transcript window plus visible Record & Replay event/window evidence.
5. For each important requirement, bug moment, workflow step, or review finding,
   record alignment confidence: `high`, `medium`, `low`, or `unresolved`.
6. Do not claim production-grade video/transcript synchronization unless the run
   has shared-clock evidence, muxed audio/video timestamps, or explicit sync
   sentinels on both narration and Record & Replay sides.
