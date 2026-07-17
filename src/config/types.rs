use std::path::PathBuf;

pub const TRANSCRIPTION_MODEL: &str = "gpt-realtime-whisper";
pub const DEFAULT_REALTIME_DELAY: &str = "high";
pub const DEFAULT_BATCH_TRANSCRIPTION_MODEL: &str = "gpt-4o-transcribe";
pub const DEFAULT_CLEANUP_MODEL: &str = "gpt-5.4-mini";
pub const DEFAULT_AUDIO_FILTER: &str =
    "highpass=f=80,lowpass=f=9000,volume=9dB,alimiter=limit=0.95";
pub const REALTIME_ENDPOINT_INTENT: &str = "transcription";
pub const MODEL: &str = TRANSCRIPTION_MODEL;
pub const SAMPLE_RATE: u32 = 24_000;
pub const DEFAULT_ROOT: &str = "/tmp/narrated-record-replay";
pub const DEFAULT_MAX_SECONDS: u64 = 1_800;

#[derive(Debug, Clone)]
pub struct Args {
    pub command: String,
    pub goal: Option<String>,
    pub root: PathBuf,
    pub skill_dir: Option<PathBuf>,
    pub session_dir: Option<PathBuf>,
    pub recording_metadata: Option<String>,
    pub recording_events: Option<String>,
    pub baseline_delay_evaluation: Option<PathBuf>,
    pub candidate_delay_evaluation: Option<PathBuf>,
    pub coverage_json: Option<PathBuf>,
    pub coverage_receipt: Option<PathBuf>,
    pub delay: String,
    pub input: String,
    pub max_seconds: Option<u64>,
    pub record_replay_status: Option<String>,
    pub microphone_capture_consent: bool,
    pub openai_postprocessing_consent: bool,
    pub custom_runtime_path_consent: bool,
    pub custom_audio_filter_consent: bool,
    pub batch_transcription_enabled: bool,
    pub cleanup_enabled: bool,
    pub batch_transcription_model: String,
    pub cleanup_model: String,
    pub audio_retention_mode: String,
    pub audio_retention_path: Option<PathBuf>,
    pub audio_filter: String,
    pub cleanup_dictionary_source: Option<PathBuf>,
    pub replay_voice_style: String,
    pub replay_voice_pace: String,
    pub replay_voice_emphasis: String,
    pub receipt_run_id: Option<String>,
    pub receipt_generated_at: Option<String>,
    pub json: bool,
}
