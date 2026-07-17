fn call_cleanup_api(
    client: &Client,
    api_key: &str,
    model: &str,
    model_input: &CleanupModelInput,
) -> Result<Value> {
    let api_url =
        std::env::var("NARRATED_REPLAY_CLEANUP_API_URL").unwrap_or_else(|_| API_URL.to_string());
    call_cleanup_api_with_url(client, &api_url, api_key, model, model_input)
}

fn call_cleanup_api_with_url(
    client: &Client,
    api_url: &str,
    api_key: &str,
    model: &str,
    model_input: &CleanupModelInput,
) -> Result<Value> {
    let api_url = validated_cleanup_api_url(api_url)?;
    let request = cleanup_request(model, model_input);
    let response = client
        .post(api_url)
        .bearer_auth(api_key)
        .json(&request)
        .send()?;
    let status = response.status();
    if !status.is_success() {
        bail!("cleanup failed with {status}; response body omitted");
    }
    let body = read_cleanup_response(response)?;
    Ok(serde_json::from_str(&body)?)
}

fn cleanup_request(model: &str, model_input: &CleanupModelInput) -> Value {
    json!({
        "model": model,
        "instructions": model_input.trusted_instructions,
        "input": [{
            "role": "user",
            "content": [{"type": "input_text", "text": model_input.untrusted_data}]
        }],
        "store": false,
        "tools": [],
        "tool_choice": "none",
        "parallel_tool_calls": false,
        "reasoning": { "effort": "low" }
    })
}

fn validated_cleanup_api_url(raw: &str) -> Result<reqwest::Url> {
    validate_cleanup_api_url(raw, cfg!(test))
}

fn validate_cleanup_api_url(raw: &str, allow_local_fixture: bool) -> Result<reqwest::Url> {
    let url = reqwest::Url::parse(raw).context("invalid cleanup API URL")?;
    let official = url.scheme() == "https"
        && url.host_str() == Some("api.openai.com")
        && url.port_or_known_default() == Some(443)
        && url.as_str() == API_URL;
    let local_fixture = allow_local_fixture
        && url.scheme() == "http"
        && matches!(url.host_str(), Some("127.0.0.1" | "localhost" | "::1"));
    if !official && !local_fixture {
        bail!("cleanup API URL must use the official OpenAI HTTPS endpoint");
    }
    Ok(url)
}

fn read_cleanup_response(mut response: reqwest::blocking::Response) -> Result<String> {
    let mut body = String::new();
    response
        .by_ref()
        .take(MAX_RESPONSE_BYTES + 1)
        .read_to_string(&mut body)?;
    if body.len() as u64 > MAX_RESPONSE_BYTES {
        bail!("cleanup response exceeds {MAX_RESPONSE_BYTES} bytes");
    }
    Ok(body)
}

#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    fn production_policy_allows_only_the_openai_responses_endpoint() {
        assert!(validate_cleanup_api_url("http://127.0.0.1:1", false).is_err());
        assert!(validate_cleanup_api_url("https://example.com/v1/responses", false).is_err());
        assert!(validate_cleanup_api_url(API_URL, false).is_ok());
    }

    #[test]
    fn test_policy_preserves_loopback_fixtures_only() {
        assert!(validate_cleanup_api_url("http://localhost:1234", true).is_ok());
        assert!(validate_cleanup_api_url("http://example.com", true).is_err());
    }
}
