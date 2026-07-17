include!("transcript_cleanup/prompt.rs");
include!("transcript_cleanup/policy.rs");
include!("transcript_cleanup/validation.rs");
include!("transcript_cleanup/binding.rs");
include!("transcript_cleanup/artifact_verification.rs");
include!("transcript_cleanup/security.rs");
include!("transcript_cleanup/api_and_dictionary.rs");
include!("transcript_cleanup/artifact.rs");
include!("transcript_cleanup/tests.rs");
include!("transcript_cleanup/test_support.rs");
include!("transcript_cleanup/prompt_tests.rs");
include!("transcript_cleanup/api_extra_tests.rs");
include!("transcript_cleanup/boundary_tests.rs");
include!("transcript_cleanup/security_fix_tests.rs");

#[cfg(test)]
static CLEANUP_ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
fn lock_cleanup_env() -> std::sync::MutexGuard<'static, ()> {
    CLEANUP_ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
