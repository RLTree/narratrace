pub(super) const MAX_REALTIME_MESSAGE_BYTES: usize = 256 * 1024;
const MAX_REALTIME_AGGREGATE_BYTES: u64 = 32 * 1024 * 1024;
const MAX_REALTIME_EVENTS: u64 = 50_000;
const MAX_CAPTURE_AUDIO_BYTES: u64 =
    crate::config::SAMPLE_RATE as u64 * 2 * crate::config::MAX_CAPTURE_SECONDS;

#[derive(Debug, Default)]
pub(super) struct CaptureQuota {
    audio_bytes: u64,
    realtime_bytes: u64,
    realtime_events: u64,
}

pub(super) fn validate_capture_duration(args: &crate::config::Args) -> anyhow::Result<()> {
    match args.max_seconds {
        Some(1..=crate::config::MAX_CAPTURE_SECONDS) => Ok(()),
        _ => anyhow::bail!(
            "capture duration must be between 1 and {} seconds",
            crate::config::MAX_CAPTURE_SECONDS
        ),
    }
}

impl CaptureQuota {
    pub(super) fn reserve_audio(&mut self, bytes: u64) -> anyhow::Result<()> {
        let next = self
            .audio_bytes
            .checked_add(bytes)
            .ok_or_else(|| anyhow::anyhow!("capture audio byte quota overflow"))?;
        if next > MAX_CAPTURE_AUDIO_BYTES {
            anyhow::bail!("capture audio exceeds {MAX_CAPTURE_AUDIO_BYTES} bytes");
        }
        self.audio_bytes = next;
        Ok(())
    }

    pub(super) fn reserve_realtime_event(&mut self, text: &str) -> anyhow::Result<()> {
        let bytes = text.len();
        if bytes > MAX_REALTIME_MESSAGE_BYTES {
            anyhow::bail!("realtime event exceeds {MAX_REALTIME_MESSAGE_BYTES} bytes");
        }
        let next_events = self
            .realtime_events
            .checked_add(1)
            .ok_or_else(|| anyhow::anyhow!("realtime event count quota overflow"))?;
        if next_events > MAX_REALTIME_EVENTS {
            anyhow::bail!("realtime capture exceeds {MAX_REALTIME_EVENTS} events");
        }
        let next_bytes = self
            .realtime_bytes
            .checked_add(bytes as u64)
            .ok_or_else(|| anyhow::anyhow!("realtime aggregate byte quota overflow"))?;
        if next_bytes > MAX_REALTIME_AGGREGATE_BYTES {
            anyhow::bail!("realtime capture exceeds {MAX_REALTIME_AGGREGATE_BYTES} event bytes");
        }
        self.realtime_events = next_events;
        self.realtime_bytes = next_bytes;
        Ok(())
    }
}

#[cfg(test)]
mod quota_tests {
    use super::*;

    #[test]
    fn audio_quota_allows_exact_limit_and_rejects_next_byte() {
        let mut quota = CaptureQuota::default();

        quota.reserve_audio(MAX_CAPTURE_AUDIO_BYTES).unwrap();

        assert!(quota.reserve_audio(1).is_err());
    }

    #[test]
    fn runtime_duration_guard_rejects_missing_or_bypassed_parser_limits() {
        let mut args = crate::config::parse_args_from(["nrr", "preflight"]).unwrap();
        args.max_seconds = None;
        assert!(validate_capture_duration(&args).is_err());

        args.max_seconds = Some(crate::config::MAX_CAPTURE_SECONDS + 1);
        assert!(validate_capture_duration(&args).is_err());
    }

    #[test]
    fn realtime_quota_enforces_message_aggregate_and_event_limits() {
        let mut oversized = CaptureQuota::default();
        assert!(
            oversized
                .reserve_realtime_event(&"x".repeat(MAX_REALTIME_MESSAGE_BYTES + 1))
                .is_err()
        );

        let mut aggregate = CaptureQuota {
            realtime_bytes: MAX_REALTIME_AGGREGATE_BYTES,
            ..CaptureQuota::default()
        };
        assert!(aggregate.reserve_realtime_event("x").is_err());

        let mut events = CaptureQuota {
            realtime_events: MAX_REALTIME_EVENTS,
            ..CaptureQuota::default()
        };
        assert!(events.reserve_realtime_event("{}").is_err());
    }
}
