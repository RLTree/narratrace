include!("batch_transcribe/prompt.rs");
include!("batch_transcribe/model_prompt.rs");
include!("batch_transcribe/binding.rs");
include!("batch_transcribe/verified.rs");
include!("batch_transcribe/security.rs");
include!("batch_transcribe/artifact.rs");
include!("batch_transcribe/api_and_audio.rs");
include!("batch_transcribe/tests.rs");
include!("batch_transcribe/chunk_tests.rs");
include!("batch_transcribe/prompt_extra_tests.rs");
include!("batch_transcribe/audio_extra_tests.rs");
include!("batch_transcribe/test_http.rs");
include!("batch_transcribe/lifecycle_tests.rs");
include!("batch_transcribe/boundary_tests.rs");
include!("batch_transcribe/security_fix_tests.rs");

#[cfg(test)]
static BATCH_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
pub(crate) fn lock_batch_env() -> std::sync::MutexGuard<'static, ()> {
    BATCH_ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
