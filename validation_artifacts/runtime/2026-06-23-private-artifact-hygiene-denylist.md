# Private Artifact Hygiene Denylist

Generated: 2026-06-23T00:42:27Z

Purpose: close the package-hygiene trust-boundary gap found by the security
reviewer after the 0.0.10-proposal sync.

Change:

- `scripts/check-package-hygiene` now rejects the documented local-private
  runtime artifacts `audio-retention.json`,
  `batch-transcription-receipt.json`, `cleanup-receipt.json`, and
  `final-transcript.timeline.jsonl`.

Focused regression probe:

```sh
mkdir -p /private/tmp/nrr-hygiene-private-artifact-probe
touch /private/tmp/nrr-hygiene-private-artifact-probe/audio-retention.json \
  /private/tmp/nrr-hygiene-private-artifact-probe/final-transcript.timeline.jsonl \
  /private/tmp/nrr-hygiene-private-artifact-probe/batch-transcription-receipt.json \
  /private/tmp/nrr-hygiene-private-artifact-probe/cleanup-receipt.json
.codex/skills/narrated-record-replay/scripts/check-package-hygiene \
  /private/tmp/nrr-hygiene-private-artifact-probe
```

Result: exited `1` as expected and reported all four disallowed paths.

Claim ceiling: this proves the denylist now catches the previously missed
private runtime filenames. It does not prove positive allowlist packaging, and
it does not prove live audio/video alignment.
