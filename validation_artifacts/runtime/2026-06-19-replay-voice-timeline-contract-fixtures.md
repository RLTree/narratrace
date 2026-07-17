---
last_edited: 2026-06-15
---

# Replay Voice Timeline Contract Fixtures

## Scope

This receipt covers a non-live `CLAIM-013` slice. It hardens
`replay-voice-parameters.json` as a typed planning artifact for future replay
behavior. It does not prove replay voice execution, replay engine consumption,
or a live demonstration.

## Changes Verified

- `replay-voice-parameters.json` now includes `timelineBindingContract`.
- Each segment binding now includes a `timelineBinding` in the
  `transcript-audio-offset-ms` clock domain.
- The artifact now carries explicit `proofObligations` and unsupported claims.
- `docs/design-docs/replay-voice-contract.md` documents the claim ceiling and
  future replay-engine preconditions.

## Commands

```sh
cargo fmt --manifest-path /Users/terrynoblin/personal-monorepo/.codex/skills/narrated-record-replay/Cargo.toml -- --check
```

Result: exit 0.

```sh
python3 -m pytest /Users/terrynoblin/personal-monorepo/tests/test_narrated_record_replay.py::test_packet_generation_from_fixture_session /Users/terrynoblin/personal-monorepo/tests/test_narrated_record_replay.py::test_packet_accepts_custom_replay_voice_parameters
```

Result: exit 0, `2 passed`.

```sh
.codex/skills/narrated-record-replay/scripts/check
```

Result: exit 0. Cargo tests passed with `0 passed`; pytest passed with
`31 passed`.

## Non-Claimed Proof

- No replay engine exists in this receipt.
- No replay behavior test was added.
- No live demonstration was run.
- `CLAIM-013` remains blocked.
