---
last_edited: 2026-06-15
---

# Redaction Policy

Generated packet artifacts are shareable only after redaction rules run. Raw transcript inputs may remain in the local session directory, but they must not be copied into durable skill files, memory, wiki, shared docs, or review artifacts by default.

## Current Generated-Artifact Scope

The helper applies pattern redaction to:

- `temporal-context.json`
- `timestamped-notes.md`
- `thought-process.md`
- `skill-refinement-packet.md`

The helper does not rewrite local raw inputs:

- `transcript.timeline.jsonl`
- `transcript.final.txt`
- `transcript.live.txt`

## Inspection Boundary

`packet-inspection.json` is the machine-readable checklist for post-packet
privacy review. Its `privacyBoundary.rawLocalOnly` section identifies raw local
transcript artifacts that must not be shared by default. Its
`privacyBoundary.distilledReviewCandidates` section identifies generated
artifacts that may be inspected as redacted review candidates, but
`allowedToShareWithoutReview` remains `false` until operator review confirms no
raw-private leakage.

The `privacyBoundary.generatedArtifactLeakScan` section performs a category-only
pattern scan over generated review candidates. It reports artifact names and
categories such as `secret-token`, `private-path`, or `email`; it must not copy
matched sensitive values into the inspection artifact.

The `privacyBoundary.rawLocalOnly` section may summarize raw local transcript
artifacts with byte counts, line counts, content fingerprints, and
category-only sensitive-pattern flags. It must not copy raw transcript text or
matched sensitive values into `packet-inspection.json`.

## Current Patterns

The current implementation redacts:

- obvious API-key-like tokens such as `sk-*`, `ghp_*`, `github_pat_*`, and `xoxb-*`;
- email-like identifiers, including when followed by common sentence punctuation;
- values attached to sensitive keys such as `password`, `token`, `secret`, `api_key`, `openai_api_key`, and `authorization`;
- private-looking local paths under user home or macOS temporary roots;
- phone-like numbers with common separators;
- JSON Web Token-shaped values and long opaque base64/hex-like tokens.

## Claim Ceiling

This is a pattern redaction and category-scan baseline, not a full privacy
guarantee. `CLAIM-010` remains blocked until the negative fixture corpus is
broader and a real generated packet from a non-toy workflow has been inspected.
