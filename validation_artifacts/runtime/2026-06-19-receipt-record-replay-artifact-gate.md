---
last_edited: 2026-06-15
---

# Receipt Record & Replay Artifact Gate

Date: 2026-06-19
Workspace: `/Users/terrynoblin/personal-monorepo`
Skill: `.codex/skills/narrated-record-replay`

## Purpose

Make `dogfood-receipt.json` fail closed when the caller supplies unusable
Record & Replay artifact paths. Live capture proof must not treat a provided
path as meaningful unless the path exists as a non-empty file.

This check uses filesystem metadata only. It does not copy Record & Replay
event content, raw transcript text, audio, secrets, or broad logs into the
receipt.

## Commands And Results

Command:

```text
cargo fmt --manifest-path .codex/skills/narrated-record-replay/Cargo.toml -- --check
```

Result:

```text
exit 0
```

Command:

```text
python3 -m pytest tests/test_narrated_record_replay_receipt.py
```

Result:

```text
2 passed in 3.27s
```

## Verified Behavior

- `receipt` keeps the successful synthetic receipt path at
  `requires-operator-review` when metadata/events files are present and
  non-empty.
- `receipt` returns `blocked` when `--recording-metadata` points to a missing
  path.
- `receipt` returns `blocked` when `--recording-events` points to an empty file.
- The receipt blocker list includes:
  - `Record & Replay metadata path must exist as a non-empty file`
  - `Record & Replay events path must exist as a non-empty file`
- The receipt artifact table records path existence, byte count, line count,
  and content fingerprint metadata only.

## Claim Ceiling

This improves the proof gate for future live dogfood receipts. It does not prove
live narrated capture, microphone transcription, packet usefulness, operator
approval, clock alignment, or replay behavior.
