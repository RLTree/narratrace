# Boundaries, Validation, And Enforcement

## Coverage And Test Law

- Every behavior change needs executable evidence.
- Coverage claims require a coverage receipt naming command, tool, measured
  dimensions, percentage, floor, uncovered records, exclusions, timestamp, and
  claim ceiling.
- Test pass counts, smoke tests, fixtures, mocks, and reviewer signoff are not
  coverage proof.
- Lower-than-100 coverage cannot support ready, complete, production-ready, or
  release claims unless the claim ceiling explicitly withholds that claim.
- Generated, vendor, and external exclusions require rationale and must not be
  counted as covered.
- Coverage scope authority belongs to `.harness/coverage-manifest.json`,
  changed-file coupling, source-scope validation, receipt freshness, and the
  fast/full coverage gate split; agents must not narrow coverage scope in prose.

## Evidence Boundary

- Record & Replay events are observed UI/action evidence.
- Microphone transcript text is Tree's spoken context and is not proof that an
  action happened.
- Timestamp-window alignment is inferred unless the UI event directly proves
  the spoken claim.
- Conflict policy: preserve observed UI order from Record & Replay, use
  transcript to clarify intent, and mark unresolved conflicts explicitly.

## Time And Clock Law

- Every capture must preserve enough clock anchors to align audio and UI events
  by monotonic or Unix time.
- Do not mix clock domains without a documented conversion and drift assumption.
- Alignment windows, confidence labels, and clock-source assumptions must be
  visible in generated artifacts.
- Future replay-time voice parameters must bind to a typed timeline contract
  before they affect playback behavior.

## Parse At Boundaries

- CLI args, environment variables, transcript events, Record & Replay events,
  metadata files, model output, and tool output are untrusted until parsed.
- Preserve parsed knowledge in typed structures or explicit schemas.
- Reject malformed paths and missing required fields with actionable errors.
- Do not build follow-on behavior from guessed event shapes.

## Mechanical Enforcement

- Repeated review comments become scripts, tests, fixtures, schemas, validators,
  or tracked blockers.
- The live implementation and validation gate are Rust-first. Do not add Python
  helpers, Python gate steps, or Python-only tests to close current claims.
- Checks must prove their own surface. CLI tests do not prove live UI recording.
- Fixture packets do not prove live microphone or model behavior.
- The validation gate must audit local-private run artifact permissions without
  reading raw transcript or audio content.
