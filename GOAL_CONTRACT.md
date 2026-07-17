---
last_edited: 2026-06-15
---

# Narrated Record & Replay Goal Contract

## Outcome

Build a serious narrated Record & Replay capability: UI event recording plus voice narration/debrief transcripts with timestamps, monotonic clock alignment, evidence compiler, redaction, conflict policy, review UI/contracts, multiple demonstrations, and future replay-time voice parameters.

## Current Scope

The initial setup pass established the skill-directory operating contract, standards binding, references, execution plan, and validation expectations. Implementation is now active under this contract, but the full recorder/transcription/review system remains unproven until the required capability claims below have fresh named evidence.

## Source Of Truth

- Skill-local files in `.codex/skills/narrated-record-replay/`.
- Current Rust helper behavior and tests.
- The installed Harness Ultragoal plugin at
  `/Users/terrynoblin/.codex/plugins/harness-ultragoal`, specifically the
  current skills, templates, standards modules, schemas, and resource map named
  in `REFERENCES.md`.
- Fresh runtime or command receipts for any completion claim.

Memory and chat are context only.

The repo-local `.codex/skills/ultragoal/SKILL.md` is not the authority for this setup.

## Non-Goals And Guardrails

- Do not replace Record & Replay.
- Do not add unrelated network/model behavior. The transcription-quality upgrade
  may call OpenAI audio transcription and Responses APIs after recording stops,
  using local-private retained audio and explicit packet-time configuration.
- Do not add non-Rust implementation dependencies. Rust crate dependencies added
  for the post-stop quality pipeline must remain covered by `scripts/check`.
- Do not record audio during ordinary validation. Bounded live dogfood may open the microphone only after explicit approval and only when Record & Replay and microphone capture start as one coordinated operation.
- Do not generate or persist raw private transcripts.
- Do not claim full integration, review UI, replay controls, or production readiness.

## Required Setup Claims

The harness ultragoal schemas require canonical claim IDs in the form `CLAIM-###`.
The legacy `NRR-*` IDs below are preserved as local aliases so older notes stay
readable, but the JSON bundle uses the canonical claim IDs.

| Claim id | Required claim | Evidence surface |
| --- | --- | --- |
| `CLAIM-001` (`NRR-SETUP-001`) | Future agents can discover the local operating contract before editing. | `AGENTS.md` and `SKILL.md` route to contract files. |
| `CLAIM-002` (`NRR-SETUP-002`) | The harness standards are bound inside the skill directory. | `AGENT_STANDARDS.md` names the upstream template and local laws. |
| `CLAIM-003` (`NRR-SETUP-003`) | Source references and authority boundaries are explicit. | `REFERENCES.md` lists local, repo, harness, and memory/wiki context boundaries. |
| `CLAIM-004` (`NRR-SETUP-004`) | Continued work has a restartable plan and TODO structure. | `EXECPLAN.md` records progress, phases, TODOs, and next dogfood workflow. |
| `CLAIM-005` (`NRR-SETUP-005`) | Validation expectations are executable and documented. | `VALIDATION.md` and `scripts/check` define the gate. |
| `CLAIM-006` (`NRR-SETUP-006`) | Privacy and evidence-boundary rules block raw transcript misuse. | `AGENT_STANDARDS.md`, `AGENTS.md`, and `SKILL.md` forbid raw private persistence. |
| `CLAIM-007` | The plugin-required ultragoal bundle exists and records current claim ceilings. | `LANE_REGISTRY.json`, `VERIFICATION_BACKLOG.json`, `COMPLETION_MANIFEST.json`, `AMENDMENTS.jsonl`, `RED_FIXTURES.json`, and lane `EXECPLAN.md` files. |

## Future Capability Claims

These claims are withheld until implemented and verified:

| Claim id | Claim | Minimum evidence before positive claim |
| --- | --- | --- |
| `CLAIM-008` (`NRR-CAPTURE-001`) | Live narrated capture works with Record & Replay and realtime transcription. | Live run receipt with Record & Replay metadata/events, narration session artifacts, and no raw-private durable leak. |
| `CLAIM-009` (`NRR-TIME-001`) | Audio and UI event timelines align correctly. | Synthetic fixture tests plus live capture showing clock anchors, alignment windows, confidence labels, and drift assumptions. |
| `CLAIM-010` (`NRR-REDACTION-001`) | Raw transcript/audio handling is privacy-safe. | Redaction policy, negative fixtures, and inspected generated packet showing only necessary distilled content. |
| `CLAIM-011` (`NRR-EVIDENCE-001`) | Evidence compiler produces useful refinement packets. | Generated packet from a real non-toy workflow, inspected for relevance, completeness, and evidence boundaries. |
| `CLAIM-012` (`NRR-REVIEW-001`) | Review UI/contracts support operator inspection. | UI/runtime proof, product-cohesion review, error/recovery states, and exact artifact paths. |
| `CLAIM-013` (`NRR-REPLAY-VOICE-001`) | Replay-time voice parameters are supported. | Typed timeline contract, replay behavior tests, and live demonstration. |

## Completion Ceiling

The maximum supported claim after setup plus the 2026-06-21 non-toy coordinated live dogfood is:

The narrated Record & Replay skill directory is standards-bound and prepared for continued ultragoal implementation, and one non-toy coordinated live dogfood produced partial runtime evidence for Record & Replay capture, microphone transcription/drain behavior, packet generation, review artifacts, and dry-run replay voice planning.

Unsupported at the current claim ceiling:

- The full recorder/transcription integration is not complete.
- Same-start proof is timestamp-proximity and thread-provenance bounded unless a durable app/tool operation id becomes available.
- Operator privacy, packet usefulness, and product-cohesion review of the 2026-06-21 generated artifacts are still owed before sharing or claim closure.
- Cross-process monotonic drift proof or explicit unavailable-capability proof is still owed.
- The review UI is not complete.
- Redaction and conflict policies are partially fixture-backed but not proven complete for real private workflows.
- Replay-time voice parameters have dry-run planning artifacts only; live replay behavior is not implemented.
- Production readiness is not proven.

## Transcription Quality Amendment

As of 2026-06-21, the normal dogfood path is realtime timing plus post-stop word
authority:

- Realtime `gpt-realtime-whisper` remains the timing spine.
- Batch transcription, cleanup, and final transcript alignment may run during
  `packet` unless disabled.
- Final alignment must not depend on Tree saying synthetic marker phrases.
  Marker labels may be used as optional anchors in scripted tests, but normal
  workflow alignment must work from cleaned utterances mapped monotonically onto
  realtime token/timing evidence.
- Realtime delay remains `high` by default until A/B dogfood shows `low`
  preserves final aligned transcript quality while improving latency. Delay is
  tuned for timing/anchor usefulness, not final word authority.
- Retained microphone audio is local-private runtime data and must not be copied
  into generated evidence packets by default.
- Offline validation may use fixtures or pre-seeded artifacts only; it does not
  prove real audio capture, real API transcription, cleanup model availability,
  or video/event alignment quality.

## Amendment Rule

Material changes to scope, claim ids, validation gates, privacy posture, or proof requirements must update this file and `EXECPLAN.md` in the same change. Do not rely on chat-only amendments.

## Review Sign-Off Amendment

As of 2026-06-21, narrated Record & Replay uses one material review-team sign-off
round instead of a multi-stage ladder. The required review round is the four
current harness personas with fresh context, full scope, `gpt-5.5`, `high`
reasoning, and same-round `SIGN_OFF` verdicts. Do not run extra model tiers,
extra pre-sign-off checks, or separate preliminary approvals for this lane
unless a concrete blocker is found and repaired.
