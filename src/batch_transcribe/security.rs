fn validated_api_url(variable: &str, default: &str) -> Result<reqwest::Url> {
    let raw = std::env::var(variable).unwrap_or_else(|_| default.to_string());
    validate_api_url(&raw, default, cfg!(test))
        .with_context(|| format!("{variable} must use the official OpenAI HTTPS endpoint"))
}

fn validate_api_url(raw: &str, expected: &str, allow_local_fixture: bool) -> Result<reqwest::Url> {
    let url = reqwest::Url::parse(raw).context("invalid transcription API URL")?;
    let official = url.scheme() == "https"
        && url.host_str() == Some("api.openai.com")
        && url.port_or_known_default() == Some(443)
        && url.as_str() == expected;
    let local_fixture = allow_local_fixture
        && url.scheme() == "http"
        && matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "::1"));
    if !official && !local_fixture {
        bail!("unauthorized API destination");
    }
    Ok(url)
}

fn read_response_text(mut response: reqwest::blocking::Response, label: &str) -> Result<String> {
    let mut body = String::new();
    response
        .by_ref()
        .take(MAX_RESPONSE_BYTES + 1)
        .read_to_string(&mut body)?;
    if body.len() as u64 > MAX_RESPONSE_BYTES {
        bail!("{label} exceeds {MAX_RESPONSE_BYTES} bytes");
    }
    Ok(body)
}

fn require_openai_postprocessing_consent(args: &Args) -> Result<()> {
    if !args.openai_postprocessing_consent {
        bail!(
            "--i-consent-to-openai-postprocessing is required before sending current-session retained audio to OpenAI"
        );
    }
    Ok(())
}

#[cfg(test)]
struct LoadedFixture {
    value: Value,
    sha256: String,
    bytes: u64,
}

#[cfg(test)]
fn load_test_fixture(
    session_dir: &Path,
    variable: &str,
    label: &str,
) -> Result<Option<LoadedFixture>> {
    let Ok(raw) = std::env::var(variable) else {
        return Ok(None);
    };
    let fixture_path = crate::safe_path::normalize_system_temp(Path::new(&raw));
    let session_dir = crate::safe_path::normalize_system_temp(session_dir);
    if !fixture_path.starts_with(&session_dir) {
        bail!("{label} must stay inside the current test session");
    }
    regular_file_metadata(&fixture_path).with_context(|| format!("{label} not readable: {raw}"))?;
    let bytes = read_regular_bytes_bounded(&fixture_path, MAX_JSON_ARTIFACT_BYTES)?;
    let value = serde_json::from_slice(&bytes)?;
    Ok(Some(LoadedFixture {
        value,
        sha256: sha256_bytes(&bytes),
        bytes: bytes.len() as u64,
    }))
}

#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    fn production_policy_rejects_non_openai_and_plaintext_endpoints() {
        assert!(validate_api_url("http://127.0.0.1:1", API_URL, false).is_err());
        assert!(
            validate_api_url(
                "https://example.com/v1/audio/transcriptions",
                API_URL,
                false
            )
            .is_err()
        );
        assert!(validate_api_url(API_URL, API_URL, false).is_ok());
    }

    #[test]
    fn test_policy_preserves_loopback_fixtures_only() {
        assert!(validate_api_url("http://127.0.0.1:1234", API_URL, true).is_ok());
        assert!(validate_api_url("http://example.com", API_URL, true).is_err());
    }
}
