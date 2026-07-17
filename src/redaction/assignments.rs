fn redact_assignment_token(token: &str) -> Option<String> {
    let mut output = String::new();
    let mut copied_through = 0;
    let mut found = false;
    for (separator, ch) in token.char_indices() {
        if !matches!(ch, '=' | ':') || separator < copied_through {
            continue;
        }
        let Some((_, key)) = assignment_key(token, separator) else {
            continue;
        };
        if !is_sensitive_key(&key) {
            continue;
        }
        let raw_end = token[separator + ch.len_utf8()..]
            .char_indices()
            .find(|(_, value)| matches!(value, '&' | '#' | ';'))
            .map(|(index, _)| separator + ch.len_utf8() + index)
            .unwrap_or(token.len());
        let value_end = trim_assignment_value_end(token, separator + ch.len_utf8(), raw_end);
        output.push_str(&token[copied_through..separator + ch.len_utf8()]);
        output.push_str("[REDACTED]");
        copied_through = value_end;
        found = true;
    }
    if !found {
        return None;
    }
    output.push_str(&token[copied_through..]);
    Some(output)
}

fn token_has_password_assignment(token: &str) -> bool {
    token.char_indices().any(|(index, ch)| {
        matches!(ch, '=' | ':')
            && assignment_key(token, index)
                .map(|(_, key)| is_password_key(&key))
                .unwrap_or(false)
    })
}

fn assignment_key(token: &str, separator: usize) -> Option<(usize, String)> {
    let prefix = &token[..separator];
    let start = prefix
        .char_indices()
        .rev()
        .find(|(_, ch)| !ch.is_ascii_alphanumeric() && !matches!(ch, '_' | '-'))
        .map(|(index, ch)| index + ch.len_utf8())
        .unwrap_or(0);
    let key = prefix[start..].replace('-', "_").to_ascii_lowercase();
    (!key.is_empty()).then_some((start, key))
}

fn trim_assignment_value_end(token: &str, start: usize, mut end: usize) -> usize {
    while end > start {
        let Some(ch) = token[..end].chars().next_back() else {
            break;
        };
        if !matches!(ch, ',' | ')' | ']' | '}' | '"' | '\'') {
            break;
        }
        end -= ch.len_utf8();
    }
    end
}

fn sensitive_key_phrase_policy(token: &str) -> Option<bool> {
    let key = token
        .trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && !matches!(ch, '_' | '-'))
        .replace('-', "_")
        .to_ascii_lowercase();
    is_sensitive_key(&key).then_some(is_password_key(&key))
}

fn is_password_key(key: &str) -> bool {
    matches!(key, "password" | "passwd" | "passphrase" | "pwd")
}

fn is_sensitive_key(key: &str) -> bool {
    matches!(
        key,
        "password"
            | "passwd"
            | "passphrase"
            | "pwd"
            | "token"
            | "secret"
            | "api_key"
            | "apikey"
            | "openai_api_key"
            | "authorization"
            | "bearer"
            | "access_token"
            | "auth_token"
            | "refresh_token"
            | "client_secret"
            | "private_key"
            | "credential"
            | "signature"
            | "sig"
            | "x_amz_signature"
            | "x_goog_signature"
            | "awsaccesskeyid"
    )
}
