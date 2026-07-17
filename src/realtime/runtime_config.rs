use std::path::Path;

const OPENAI_REALTIME_URL: &str = "wss://api.openai.com/v1/realtime?intent=transcription";
const TRUSTED_FFMPEG_PATHS: [&str; 3] = [
    "/opt/homebrew/bin/ffmpeg",
    "/usr/local/bin/ffmpeg",
    "/usr/bin/ffmpeg",
];

#[derive(Clone, Copy)]
struct RuntimeConfig<'a> {
    realtime_url: &'a str,
    ffmpeg_binary: &'a Path,
}

impl RuntimeConfig<'static> {
    fn production() -> std::io::Result<Self> {
        Ok(Self {
            realtime_url: OPENAI_REALTIME_URL,
            ffmpeg_binary: ffmpeg_binary()?,
        })
    }
}

#[cfg(test)]
impl<'a> RuntimeConfig<'a> {
    fn for_test(realtime_url: &'a str, ffmpeg_binary: &'a Path) -> Self {
        Self {
            realtime_url,
            ffmpeg_binary,
        }
    }
}

pub(crate) fn ffmpeg_binary() -> std::io::Result<&'static Path> {
    for candidate in TRUSTED_FFMPEG_PATHS {
        let path = Path::new(candidate);
        if path.is_file() {
            return Ok(path);
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::NotFound,
        format!(
            "ffmpeg was not found at a trusted absolute path: {}",
            TRUSTED_FFMPEG_PATHS.join(", ")
        ),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn production_runtime_ignores_legacy_environment_overrides() {
        let _guard = ENV_LOCK.lock().unwrap();
        unsafe {
            std::env::set_var("NARRATED_REPLAY_REALTIME_URL", "ws://127.0.0.1:1234");
            std::env::set_var("NARRATED_REPLAY_FFMPEG", "/tmp/fake-ffmpeg");
        }

        let config = RuntimeConfig::production().unwrap();

        assert_eq!(config.realtime_url, OPENAI_REALTIME_URL);
        assert!(config.realtime_url.starts_with("wss://api.openai.com/"));
        assert!(config.ffmpeg_binary.is_absolute());
        assert_ne!(config.ffmpeg_binary, Path::new("/tmp/fake-ffmpeg"));
        unsafe {
            std::env::remove_var("NARRATED_REPLAY_REALTIME_URL");
            std::env::remove_var("NARRATED_REPLAY_FFMPEG");
        }
    }

    #[test]
    fn test_runtime_accepts_explicit_local_dependencies() {
        let path = Path::new("/private/tmp/fake-ffmpeg");
        let config = RuntimeConfig::for_test("ws://127.0.0.1:1234", path);

        assert_eq!(config.realtime_url, "ws://127.0.0.1:1234");
        assert_eq!(config.ffmpeg_binary, path);
    }
}
