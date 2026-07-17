const PHRASE_TOKEN_CAP: u8 = 8;

pub fn redact_text(value: &str) -> String {
    let phone_redacted = redact_phone_sequences(value);
    let mut output = String::with_capacity(phone_redacted.len());
    let mut token = String::new();
    let mut secret_value = SecretValueState::None;

    for ch in phone_redacted.chars() {
        if ch.is_whitespace() {
            flush_token(&mut output, &mut token, &mut secret_value);
            output.push(ch);
        } else {
            token.push(ch);
        }
    }
    flush_token(&mut output, &mut token, &mut secret_value);
    output
}

pub fn sensitive_categories(value: &str) -> Vec<&'static str> {
    let mut categories = Vec::new();
    if contains_phone_sequence(value) {
        categories.push("phone-number");
    }
    let mut token = String::new();
    let mut secret_value = SecretValueState::None;

    for ch in value.chars() {
        if ch.is_whitespace() {
            collect_sensitive_category(&mut categories, &token, &mut secret_value);
            token.clear();
        } else {
            token.push(ch);
        }
    }
    collect_sensitive_category(&mut categories, &token, &mut secret_value);
    categories
}

fn collect_sensitive_category(
    categories: &mut Vec<&'static str>,
    token: &str,
    state: &mut SecretValueState,
) {
    if token.is_empty() {
        return;
    }
    if let Some(category) = sensitive_category(token, *state != SecretValueState::None)
        && !categories.contains(&category)
    {
        categories.push(category);
    }
    *state = next_secret_value_state(token, *state);
}

fn sensitive_category(token: &str, sensitive_previous_token: bool) -> Option<&'static str> {
    if redact_assignment_token(token).is_some() {
        return Some("sensitive-key-value");
    }
    if looks_like_secret_token(token) || looks_like_aws_access_key_id(token) {
        return Some("secret-token");
    }
    if sensitive_previous_token && !token_is_separator(token) {
        return Some("sensitive-key-value");
    }
    if looks_like_email(token) {
        return Some("email");
    }
    if looks_like_private_path(token) {
        return Some("private-path");
    }
    if looks_like_jwt(token) {
        return Some("jwt-token");
    }
    looks_like_long_opaque_token(token).then_some("opaque-token")
}

#[derive(Clone, Copy, PartialEq)]
enum SecretValueState {
    None,
    OneToken { phrase_allowed: bool },
    Phrase { remaining: u8 },
}

fn flush_token(output: &mut String, token: &mut String, state: &mut SecretValueState) {
    if token.is_empty() {
        return;
    }
    let redact_from_key = *state != SecretValueState::None && !token_is_separator(token);
    output.push_str(&redact_token(token, redact_from_key));
    *state = next_secret_value_state(token, *state);
    token.clear();
}

fn next_secret_value_state(token: &str, state: SecretValueState) -> SecretValueState {
    if let Some(phrase_allowed) = sensitive_key_phrase_policy(token) {
        return SecretValueState::OneToken { phrase_allowed };
    }
    if token_has_password_assignment(token) {
        return if token_ends_phrase(token) {
            SecretValueState::None
        } else {
            SecretValueState::Phrase {
                remaining: PHRASE_TOKEN_CAP.saturating_sub(1),
            }
        };
    }
    match state {
        SecretValueState::OneToken {
            phrase_allowed: true,
        } if token_is_separator(token) => SecretValueState::Phrase {
            remaining: PHRASE_TOKEN_CAP,
        },
        SecretValueState::OneToken {
            phrase_allowed: true,
        } if token_is_phrase_connector(token) => SecretValueState::Phrase {
            remaining: PHRASE_TOKEN_CAP,
        },
        SecretValueState::Phrase { remaining } if !token_ends_phrase(token) && remaining > 1 => {
            SecretValueState::Phrase {
                remaining: remaining - 1,
            }
        }
        SecretValueState::OneToken { .. } if token_is_separator(token) => state,
        _ => SecretValueState::None,
    }
}

fn redact_token(token: &str, redact_because_previous_token: bool) -> String {
    if let Some(redacted) = redact_assignment_token(token) {
        return redacted;
    }
    if sensitive_key_phrase_policy(token).is_some() {
        return token.to_string();
    }
    if looks_like_secret_token(token) || looks_like_aws_access_key_id(token) {
        return preserve_edge_punctuation(token, "[REDACTED_SECRET]");
    }
    if redact_because_previous_token && !token_is_separator(token) {
        return preserve_edge_punctuation(token, "[REDACTED]");
    }
    if looks_like_email(token) {
        return preserve_edge_punctuation(token, "[REDACTED_EMAIL]");
    }
    if looks_like_private_path(token) {
        return preserve_edge_punctuation(token, "[REDACTED_PATH]");
    }
    if looks_like_jwt(token) || looks_like_long_opaque_token(token) {
        return preserve_edge_punctuation(token, "[REDACTED_TOKEN]");
    }
    token.to_string()
}
