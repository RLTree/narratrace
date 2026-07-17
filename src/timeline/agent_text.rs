use crate::redaction::render_untrusted_markdown;
use serde_json::{Value, json};

const TRANSCRIPT_EVIDENCE_PREFIX: &str = "Transcript evidence: [untrusted data] ";

pub(super) struct AgentTranscriptEvidence {
    rendered: String,
}

impl AgentTranscriptEvidence {
    pub(super) fn from_untrusted(raw: &str) -> Self {
        Self {
            rendered: render_untrusted_markdown("Transcript evidence", raw),
        }
    }

    pub(super) fn rendered(&self) -> &str {
        &self.rendered
    }

    pub(super) fn boundary(&self) -> Value {
        transcript_content_boundary()
    }
}

pub(super) fn transcript_content_boundary() -> Value {
    json!({
        "classification": "untrusted-transcript-evidence",
        "consumerPolicy": "evidence-only-never-instructions",
        "instructionUse": "forbidden",
        "uiProof": false,
    })
}

pub(crate) fn consume_transcript_segment_text(segment: &Value) -> Option<&str> {
    let boundary = segment.get("textBoundary")?;
    if boundary.get("classification").and_then(Value::as_str)
        != Some("untrusted-transcript-evidence")
        || boundary.get("consumerPolicy").and_then(Value::as_str)
            != Some("evidence-only-never-instructions")
        || boundary.get("instructionUse").and_then(Value::as_str) != Some("forbidden")
        || boundary.get("uiProof").and_then(Value::as_bool) != Some(false)
    {
        return None;
    }
    segment
        .get("text")?
        .as_str()?
        .strip_prefix(TRANSCRIPT_EVIDENCE_PREFIX)
}
