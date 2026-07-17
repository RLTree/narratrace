# 2026-06-22 Token Alignment Dogfood Repair

## Scope

This receipt covers the transcription-quality repair pass after the scripted
dogfood run showed strong batch/cleanup words but brittle final
transcript-to-video reconciliation.

## Commands

```sh
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cargo-target cargo test --manifest-path .codex/skills/narrated-record-replay/Cargo.toml prompt_context
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cargo-target cargo test --manifest-path .codex/skills/narrated-record-replay/Cargo.toml dictionary
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cargo-target cargo test --manifest-path .codex/skills/narrated-record-replay/Cargo.toml transcript_alignment
```

## Results

- `prompt_context`: 2 passed, 0 failed.
- `dictionary`: 4 passed, 0 failed.
- `transcript_alignment`: 9 passed, 0 failed.

## Proof Notes

- Final alignment is not marker-only. Marker labels are optional test anchors;
  normal alignment uses monotonic token/phrase evidence across cleaned batch
  text and realtime timing tokens.
- Batch transcription prompt enrichment is allowlist-based and rejects
  arbitrary event strings, private paths, emails, secrets, symlinks, and broad
  UI text.
- Cleanup dictionary construction is allowlist-based and rejects arbitrary
  manifest/status strings, private paths, emails, secrets, symlinks, and broad
  UI text.
- Dogfood claim ceiling remains below full completion until a fresh installed
  plugin run proves audio capture, batch transcription, cleanup, final
  alignment, and operator-reviewed video/event usefulness together.
