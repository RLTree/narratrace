# Security, Reliability, And Product Cohesion

## Privacy And Redaction

- Raw audio, raw transcripts, private workflow details, secrets, credentials,
  and broad logs are local-private by default.
- Durable outputs must be distilled, redacted, and scoped to what future agents
  need.
- Do not write private transcripts to memory, wiki, shared docs, Slack, email,
  git, or examples unless Tree explicitly asks and the content is privacy-safe.
- Fixtures must use synthetic or redacted content.

## Runtime Proof

- Default setup validation can avoid audio and network calls, but feature
  completion cannot.
- A recorder feature claim needs live Record & Replay plus narration evidence
  on a real workflow.
- A review UI claim needs UI evidence, product-cohesion inspection, and
  recovery/error-state proof.
- An evidence compiler claim needs generated artifacts inspected for usefulness,
  not just command success.

## Product Cohesion

- Direct narrated plugin invocation starts capture first. Infer target surface
  and output from replay plus transcript before asking follow-up questions.
- The default bounded capture length is 30 minutes unless the user supplied a
  duration.
- Automatic microphone capture starts with Record & Replay. Do not make the user
  start one surface and then the other for normal dogfood.
- `auto` microphone input must avoid iPhone and virtual devices. Prefer current
  non-iPhone macOS default, then AirPods, then MacBook microphone.

## Product Fitness

- Product-impacting success, readiness, release, daily-driver, or material
  product signoff claims require Product Fitness proof separate from Product
  Cohesion.
- Product Fitness proof must bind audience, job, context, outcome,
  quality-in-use, accessibility, cognitive-load, recovery, continuance when
  claimed, and claim ceiling.
- Reviewer agreement, install success, smoke tests, test pass counts, fixture
  pass counts, package publication, first use, feature delivery, and Product
  Cohesion receipts alone do not prove product success.
