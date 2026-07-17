#[allow(clippy::too_many_arguments)]
fn post_commit_drain_receipt(
    waited_ms: u64,
    post_commit_messages: u64,
    post_commit_completed_segments: u64,
    post_commit_errors: Vec<String>,
    audio_chunks_sent: u64,
    audio_bytes_sent: u64,
    audio_commits_sent: u64,
    audio_bytes_since_commit: u64,
    realtime_messages: u64,
    realtime_delta_events: u64,
    realtime_completed_segments: u64,
    realtime_error_events: u64,
    audio_filter: &str,
    final_commit_status: &str,
) -> serde_json::Value {
    json!({
        "schema": "narrated-record-replay.post-commit-drain.v1",
        "timeoutMs": 5000,
        "waitedMs": waited_ms,
        "messages": post_commit_messages,
        "completedSegments": post_commit_completed_segments,
        "errors": post_commit_errors,
        "finalCommit": {
            "status": final_commit_status,
            "postSendMessagesObserved": post_commit_messages,
            "postSendCompletedSegmentsObserved": post_commit_completed_segments
        },
        "captureStats": {
            "audioChunksSent": audio_chunks_sent,
            "audioBytesSent": audio_bytes_sent,
            "audioCommitsSent": audio_commits_sent,
            "audioBytesPendingFinalCommit": audio_bytes_since_commit,
            "realtimeMessagesObserved": realtime_messages + post_commit_messages,
            "realtimeDeltaEventsObserved": realtime_delta_events,
            "realtimeCompletedSegmentsObserved": realtime_completed_segments,
            "realtimeErrorEventsObserved": realtime_error_events,
            "audioFilter": audio_filter
        },
        "claimCeiling": "metadata-only drain receipt; raw transcript content stays in session-private transcript artifacts"
    })
}
