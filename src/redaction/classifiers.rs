fn looks_like_secret_token(token: &str) -> bool {
    let core = token.trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_' && ch != '-');
    let lower = core.to_ascii_lowercase();
    (lower.starts_with("sk-") && core.len() >= 12)
        || (lower.starts_with("ghp_") && core.len() >= 12)
        || (lower.starts_with("github_pat_") && core.len() >= 16)
        || (lower.starts_with("xoxb-") && core.len() >= 12)
}

fn looks_like_aws_access_key_id(token: &str) -> bool {
    let core = token.trim_matches(|ch: char| !ch.is_ascii_alphanumeric());
    core.len() == 20
        && (core.starts_with("AKIA") || core.starts_with("ASIA"))
        && core
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit())
}

fn looks_like_email(token: &str) -> bool {
    let core = email_core(token);
    let Some((local, domain)) = core.split_once('@') else {
        return false;
    };
    !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
}

fn email_core(token: &str) -> &str {
    token
        .trim_matches(|ch: char| {
            !ch.is_ascii_alphanumeric() && !matches!(ch, '@' | '.' | '_' | '%' | '+' | '-')
        })
        .trim_end_matches(|ch| matches!(ch, '.' | ',' | ';' | ':' | '!' | '?'))
}

fn looks_like_private_path(token: &str) -> bool {
    let core = token.trim_matches(|ch: char| {
        ch.is_ascii_punctuation() && !matches!(ch, '/' | '~' | '.' | '_' | '-')
    });
    core.starts_with("/Users/")
        || core.starts_with("/home/")
        || core.starts_with("~/")
        || core.starts_with("/private/var/")
        || core.starts_with("/var/folders/")
        || core.starts_with("/private/tmp/")
}

fn contains_phone_sequence(value: &str) -> bool {
    phone_sequence_spans(value).next().is_some()
}

fn redact_phone_sequences(value: &str) -> String {
    let spans = phone_sequence_spans(value).collect::<Vec<_>>();
    if spans.is_empty() {
        return value.to_string();
    }
    let mut output = String::with_capacity(value.len());
    let mut copied_through = 0;
    for (start, end) in spans {
        output.push_str(&value[copied_through..start]);
        output.push_str("[REDACTED_PHONE]");
        copied_through = end;
    }
    output.push_str(&value[copied_through..]);
    output
}

fn phone_sequence_spans(value: &str) -> impl Iterator<Item = (usize, usize)> + '_ {
    let mut spans = Vec::new();
    let mut cursor = 0;
    while cursor < value.len() {
        let Some((offset, ch)) = value[cursor..]
            .char_indices()
            .find(|(_, ch)| ch.is_ascii_digit() || matches!(ch, '+' | '('))
        else {
            break;
        };
        let start = cursor + offset;
        let previous_is_alphanumeric = value[..start]
            .chars()
            .next_back()
            .map(|previous| previous.is_ascii_alphanumeric())
            .unwrap_or(false);
        let mut raw_end = start;
        for (relative, candidate) in value[start..].char_indices() {
            if candidate.is_ascii_digit()
                || matches!(candidate, '+' | '-' | '(' | ')' | '.' | ' ' | '\t')
            {
                raw_end = start + relative + candidate.len_utf8();
            } else {
                break;
            }
        }
        let end = start
            + value[start..raw_end]
                .trim_end_matches(|candidate: char| {
                    candidate.is_whitespace() || matches!(candidate, '+' | '-' | '(' | '.')
                })
                .len();
        let digits = value[start..end]
            .chars()
            .filter(|candidate| candidate.is_ascii_digit())
            .count();
        let next_is_alphanumeric = value[end..]
            .chars()
            .next()
            .map(|next| next.is_ascii_alphanumeric())
            .unwrap_or(false);
        if !previous_is_alphanumeric && !next_is_alphanumeric && (10..=15).contains(&digits) {
            spans.push((start, end));
            cursor = end;
        } else {
            cursor = start + ch.len_utf8();
        }
    }
    spans.into_iter()
}

fn looks_like_jwt(token: &str) -> bool {
    let core = token.trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_' && ch != '-');
    let parts = core.split('.').collect::<Vec<_>>();
    parts.len() == 3
        && parts.iter().all(|part| part.len() >= 8)
        && parts.iter().all(|part| {
            part.chars()
                .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
        })
}

fn looks_like_long_opaque_token(token: &str) -> bool {
    let core = token.trim_matches(|ch: char| {
        !ch.is_ascii_alphanumeric() && !matches!(ch, '_' | '-' | '+' | '/' | '=')
    });
    if core.len() < 32 {
        return false;
    }
    let allowed = core
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '+' | '/' | '='));
    if !allowed {
        return false;
    }
    let digit_count = core.chars().filter(|ch| ch.is_ascii_digit()).count();
    let uppercase_count = core.chars().filter(|ch| ch.is_ascii_uppercase()).count();
    let symbol_count = core
        .chars()
        .filter(|ch| matches!(ch, '_' | '-' | '+' | '/' | '='))
        .count();
    digit_count > 0 || uppercase_count >= 4 || symbol_count > 0
}

fn token_is_separator(token: &str) -> bool {
    token.chars().all(|ch| matches!(ch, ':' | '=' | '-'))
}

fn token_is_phrase_connector(token: &str) -> bool {
    matches!(
        token
            .trim_matches(|ch: char| !ch.is_ascii_alphabetic())
            .to_ascii_lowercase()
            .as_str(),
        "is" | "was" | "equals"
    )
}

fn token_ends_phrase(token: &str) -> bool {
    token.ends_with(['.', ',', ';', '!', '?'])
}

fn preserve_edge_punctuation(token: &str, replacement: &str) -> String {
    let mut start = 0;
    let mut end = token.len();
    for (index, ch) in token.char_indices() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '@' | '/' | '~') {
            start = index;
            break;
        }
    }
    for (index, ch) in token.char_indices().rev() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | '@' | '/' | '~') {
            end = index + ch.len_utf8();
            break;
        }
    }
    format!("{}{}{}", &token[..start], replacement, &token[end..])
}
