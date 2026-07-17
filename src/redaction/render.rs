const UNTRUSTED_MARKDOWN_MAX_CHARS: usize = 4_096;

pub fn render_untrusted_markdown(label: &str, value: &str) -> String {
    let mut bounded = String::with_capacity(value.len().min(UNTRUSTED_MARKDOWN_MAX_CHARS));
    let mut truncated = false;
    for (index, ch) in value.chars().enumerate() {
        if index == UNTRUSTED_MARKDOWN_MAX_CHARS {
            truncated = true;
            break;
        }
        bounded.push(ch);
    }
    let redacted = redact_text(&bounded);
    let mut rendered = String::with_capacity(redacted.len().min(UNTRUSTED_MARKDOWN_MAX_CHARS));
    let mut needs_space = false;

    for ch in redacted.chars() {
        if ch.is_whitespace() {
            needs_space = !rendered.is_empty();
            continue;
        }
        if needs_space {
            rendered.push(' ');
            needs_space = false;
        }
        if matches!(
            ch,
            '\\' | '`'
                | '*'
                | '_'
                | '{'
                | '}'
                | '['
                | ']'
                | '<'
                | '>'
                | '#'
                | '+'
                | '-'
                | '!'
                | '|'
                | '('
                | ')'
        ) {
            rendered.push('\\');
        }
        rendered.push(ch);
    }
    if truncated {
        rendered.push_str("...");
    }

    format!("{label}: [untrusted data] {rendered}")
}
