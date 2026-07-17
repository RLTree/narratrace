#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ReceiptState {
    pub status: String,
    pub reason: String,
    valid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct TranscriptQualityState {
    pub batch: ReceiptState,
    pub cleanup: ReceiptState,
    pub final_receipt: ReceiptState,
}

pub(super) struct FinalAlignmentReviewState {
    pub status: String,
    pub word_authority: String,
    pub unresolved_mismatches: u64,
}

pub(super) fn final_alignment_review_state(session_dir: &Path) -> FinalAlignmentReviewState {
    let value =
        read_json(&session_dir.join("final-transcript-alignment.json")).unwrap_or(Value::Null);
    FinalAlignmentReviewState {
        status: value
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("not-generated")
            .to_string(),
        word_authority: value
            .get("wordAuthority")
            .and_then(Value::as_str)
            .unwrap_or("realtime-raw")
            .to_string(),
        unresolved_mismatches: value
            .get("unresolvedMismatches")
            .and_then(Value::as_u64)
            .unwrap_or(0),
    }
}

impl TranscriptQualityState {
    pub fn chain_label(&self) -> String {
        format!(
            "batch={} ({}) | cleanup={} ({}) | final-receipt={} ({})",
            self.batch.status,
            self.batch.reason,
            self.cleanup.status,
            self.cleanup.reason,
            self.final_receipt.status,
            self.final_receipt.reason
        )
    }

    pub fn is_complete(&self) -> bool {
        [&self.batch, &self.cleanup, &self.final_receipt]
            .iter()
            .all(|state| state.valid)
    }
}

pub(super) fn transcript_quality_state(session_dir: &Path) -> TranscriptQualityState {
    TranscriptQualityState {
        batch: receipt_state(
            &session_dir.join("batch-transcription-receipt.json"),
            "narrated-record-replay.batch-transcription-receipt.v1",
        ),
        cleanup: receipt_state(
            &session_dir.join("cleanup-receipt.json"),
            "narrated-record-replay.cleanup-receipt.v1",
        ),
        final_receipt: receipt_state(
            &session_dir.join("final-transcript-alignment-receipt.json"),
            "narrated-record-replay.final-transcript-alignment-receipt.v1",
        ),
    }
}

fn receipt_state(path: &Path, expected_schema: &str) -> ReceiptState {
    let receipt = read_json(path).unwrap_or(Value::Null);
    ReceiptState {
        status: receipt
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("not-generated")
            .to_string(),
        reason: receipt
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or("not-provided")
            .to_string(),
        valid: receipt.get("schema").and_then(Value::as_str) == Some(expected_schema)
            && receipt.get("status").and_then(Value::as_str) == Some("completed"),
    }
}
