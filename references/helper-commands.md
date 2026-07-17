# Helper Commands

Use the installed plugin manifest in app-visible sessions:

```sh
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cargo-target cargo run --manifest-path /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/Cargo.toml -- prepare-coordinated-session
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cargo-target cargo run --manifest-path /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/Cargo.toml -- stop --session-dir "/tmp/narrated-record-replay/<run>"
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cargo-target cargo run --manifest-path /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/Cargo.toml -- packet --session-dir "/tmp/narrated-record-replay/<run>" --recording-metadata "<metadata-path>" --recording-events "<events-path>" --i-consent-to-openai-postprocessing
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cargo-target cargo run --manifest-path /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/Cargo.toml -- inspect --session-dir "/tmp/narrated-record-replay/<run>"
CARGO_TARGET_DIR=/private/tmp/narrated-record-replay-cargo-target cargo run --manifest-path /Users/terrynoblin/.codex/plugins/narrated-record-replay/skills/narrated-record-replay/Cargo.toml -- receipt --session-dir "/tmp/narrated-record-replay/<run>" --recording-metadata "<metadata-path>" --recording-events "<events-path>"
```

Other useful subcommands:

- `preflight --json --record-replay-status idle`
- `validate --json`
- `parent-operation-receipt`
- `delay-eval`
- `delay-compare`
- `replay-voice-preview`

Do not use standalone `start` as live Record & Replay proof. Live proof must
prepare a coordinated session and then launch Record & Replay plus microphone
capture from the same parent/orchestrator operation.
