enum FinalCommitStatus {
    NotNeeded,
    AwaitingAcknowledgement,
    Acknowledged,
    SendFailed(String),
    Unacknowledged(String),
}

struct FinalCommitProgress {
    status: FinalCommitStatus,
    audio_bytes_pending: u64,
    audio_commits_sent: u64,
}

impl FinalCommitProgress {
    fn error(&self) -> Option<&str> {
        match &self.status {
            FinalCommitStatus::SendFailed(error) | FinalCommitStatus::Unacknowledged(error) => {
                Some(error)
            }
            _ => None,
        }
    }

    fn errors(&self) -> Vec<String> {
        self.error().into_iter().map(str::to_owned).collect()
    }

    fn finish(
        &mut self,
        post_send_completed_segments: u64,
        errors: &mut Vec<String>,
    ) -> Option<String> {
        if matches!(&self.status, FinalCommitStatus::AwaitingAcknowledgement) {
            if post_send_completed_segments > 0 {
                self.audio_commits_sent += 1;
                self.audio_bytes_pending = 0;
                self.status = FinalCommitStatus::Acknowledged;
            } else {
                let error =
                    "final audio commit was not acknowledged by a post-send completed event"
                        .to_string();
                errors.push(error.clone());
                self.status = FinalCommitStatus::Unacknowledged(error);
            }
        }
        self.error().map(str::to_owned)
    }

    fn status(&self) -> &'static str {
        match &self.status {
            FinalCommitStatus::NotNeeded => "not-needed",
            FinalCommitStatus::AwaitingAcknowledgement => "awaiting-acknowledgement",
            FinalCommitStatus::Acknowledged => "acknowledged",
            FinalCommitStatus::SendFailed(_) => "send-failed",
            FinalCommitStatus::Unacknowledged(_) => "unacknowledged",
        }
    }
}

async fn send_final_audio_commit<S>(
    write: &mut S,
    audio_bytes_pending: u64,
    audio_commits_sent: u64,
) -> FinalCommitProgress
where
    S: futures_util::Sink<Message> + Unpin,
    S::Error: std::fmt::Display,
{
    let status = if audio_bytes_pending < MINIMUM_COMMIT_BYTES {
        FinalCommitStatus::NotNeeded
    } else {
        match write
            .send(Message::Text(
                json!({"type":"input_audio_buffer.commit"})
                    .to_string()
                    .into(),
            ))
            .await
        {
            Ok(()) => FinalCommitStatus::AwaitingAcknowledgement,
            Err(error) => {
                FinalCommitStatus::SendFailed(format!("final audio commit send failed: {error}"))
            }
        }
    };
    FinalCommitProgress {
        status,
        audio_bytes_pending,
        audio_commits_sent,
    }
}
