# Product Cohesion

## Surface

The product surface is the narrated Record & Replay plugin skill suite as used
by Codex agents and Tree. Capture first is the product rule. The primary
journey is:

1. User invokes the core plugin or a companion narrated skill.
2. Agent starts coordinated Record & Replay plus microphone capture without
   pre-recording intake.
3. Agent packages local-private audio/transcript/event artifacts.
4. Agent inspects alignment, privacy, and usefulness artifacts.
5. Agent routes to workflow capture, feature implementation, bug report, review
   debrief, or a custom plan.
6. Agent asks follow-up questions only after packet review if evidence is
   ambiguous.

## Product Promise

The plugin lets users show and narrate what they want, then lets agents recover
the task, evidence, and next action without forcing a form-style intake before
recording.

## Human Attention Policy

Agents handle route selection, default duration, consent flags for bounded
plugin capture, microphone fallback, packet inspection, and alignment review.
Human attention is required for raw artifact export, secrets, external sharing,
ambiguous target surfaces after packet review, and unsupported proof claims.

## Claim Ceiling

Current static and fixture proof supports an agent-first, packageable skill
suite with local privacy and validation gates. Production-grade live
transcript/video synchronization still requires live dogfood evidence with
shared-clock or sync-sentinel proof and operator-reviewed packet usefulness.
