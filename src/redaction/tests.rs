use super::*;

#[test]
fn redacts_each_sensitive_token_class_with_punctuation() {
    let input = concat!(
        "email tree@example.com, ",
        "path /Users/tree/private.txt ",
        "phone (555)-123-4567 ",
        "key sk-proj-abcdefghijklmnopqrstuvwxyz ",
        "jwt abcdefghi.abcdefghj.abcdefghk ",
        "opaque ABCDEFGHIJKLMNOPQRSTUVWXYZ123456 ",
        "assignment api_key=secretvalue"
    );

    let redacted = redact_text(input);

    assert!(redacted.contains("[REDACTED_EMAIL],"));
    assert!(redacted.contains("[REDACTED_PATH]"));
    assert!(redacted.contains("[REDACTED_PHONE]"));
    assert!(redacted.contains("[REDACTED_SECRET]"));
    assert!(redacted.contains("[REDACTED_TOKEN]"));
    assert!(redacted.contains("api_key=[REDACTED]"));
}

#[test]
fn redacts_value_after_standalone_sensitive_key_but_not_separator() {
    let redacted = redact_text("password : hunter2 then token = abc123");

    assert!(redacted.contains("password : [REDACTED]"));
    assert!(redacted.contains("token = [REDACTED]"));
}

#[test]
fn sensitive_categories_are_unique_and_cover_all_classes() {
    let categories = sensitive_categories(concat!(
        "api_key=short sk-proj-abcdefghijklmnopqrstuvwxyz ",
        "tree@example.com /private/var/folders/abc ",
        "555-123-4567 abcdefghi.abcdefghj.abcdefghk ",
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ123456"
    ));

    for expected in [
        "sensitive-key-value",
        "email",
        "private-path",
        "phone-number",
        "jwt-token",
        "opaque-token",
    ] {
        assert!(categories.contains(&expected), "{expected}");
    }
    assert_eq!(
        categories.iter().filter(|value| **value == "email").count(),
        1
    );
}

#[test]
fn classifiers_reject_near_misses() {
    assert!(!looks_like_email("tree@example"));
    assert!(!looks_like_jwt("short.parts.no"));
    assert!(!looks_like_long_opaque_token(
        "lowercasewithoutdigitsorsymbolsabcdefghi"
    ));
    assert_eq!(redact_text("ordinary narration"), "ordinary narration");
}

#[test]
fn redacts_canonical_short_credential_and_reports_its_category() {
    let credential = "AKIAIOSFODNN7EXAMPLE";

    assert_eq!(redact_text(credential), "[REDACTED_SECRET]");
    assert_eq!(sensitive_categories(credential), vec!["secret-token"]);
    assert!(!looks_like_aws_access_key_id("akiaiosfodnn7example"));
    assert!(!looks_like_aws_access_key_id("ABCDEFGHIJKLMNOPQRST"));
}

#[test]
fn redacts_explicit_multi_word_passphrase_without_consuming_next_sentence() {
    let redacted =
        redact_text("password is correct horse battery staple. Ordinary narration remains.");

    assert_eq!(
        redacted,
        "password [REDACTED] [REDACTED] [REDACTED] [REDACTED] [REDACTED]. Ordinary narration remains."
    );
    assert_eq!(
        redact_text("password required for login"),
        "password [REDACTED] for login"
    );
    assert_eq!(
        sensitive_categories("password : correct horse battery staple"),
        vec!["sensitive-key-value"]
    );
}

#[test]
fn redacts_contiguous_and_grouped_phone_numbers_without_joining_short_counts() {
    assert_eq!(
        redact_text("call 5551234567 or 555 123 4567 today"),
        "call [REDACTED_PHONE] or [REDACTED_PHONE] today"
    );
    assert_eq!(
        sensitive_categories("call +1 (555) 123-4567"),
        vec!["phone-number"]
    );
    assert_eq!(
        redact_text("counts 123 456 and 2026-07-16"),
        "counts 123 456 and 2026-07-16"
    );
}

#[test]
fn redacts_multi_word_passphrases_after_all_assignment_forms() {
    for input in [
        "password : correct horse battery staple. Visible again.",
        "password = correct horse battery staple. Visible again.",
        "password=correct horse battery staple. Visible again.",
        "passphrase:correct horse battery staple. Visible again.",
    ] {
        let redacted = redact_text(input);
        assert!(!redacted.contains("correct"), "{redacted}");
        assert!(!redacted.contains("horse"), "{redacted}");
        assert!(!redacted.contains("battery"), "{redacted}");
        assert!(!redacted.contains("staple"), "{redacted}");
        assert!(redacted.ends_with("Visible again."), "{redacted}");
    }
}

#[test]
fn embedded_url_and_header_assignments_redact_short_secret_values() {
    let input = concat!(
        "https://example.test/callback?token=short&ok=yes ",
        "https://example.test/object?X-Amz-Signature=abc123#fragment ",
        "Authorization:BearerValue api_key=short"
    );
    let redacted = redact_text(input);

    assert_eq!(
        redacted,
        concat!(
            "https://example.test/callback?token=[REDACTED]&ok=yes ",
            "https://example.test/object?X-Amz-Signature=[REDACTED]#fragment ",
            "Authorization:[REDACTED] api_key=[REDACTED]"
        )
    );
    assert_eq!(sensitive_categories(input), vec!["sensitive-key-value"]);
}

#[test]
fn untrusted_markdown_renderer_labels_bounds_and_neutralizes_structure() {
    let rendered = render_untrusted_markdown(
        "Transcript segment",
        "first line\n## injected [link](https://example.test) password=secret",
    );

    assert!(rendered.starts_with("Transcript segment: [untrusted data] "));
    assert!(!rendered.contains("\n"));
    assert!(!rendered.contains("## injected"));
    assert!(!rendered.contains("secret"));
    assert!(rendered.contains("\\#\\# injected"));

    let long = render_untrusted_markdown("Goal", &"x".repeat(5_000));
    assert!(long.ends_with("..."));
    assert!(long.len() < 4_200);
}

#[test]
fn private_tmp_paths_are_classified_and_redacted() {
    let value = "/private/tmp/unrelated/private-project.txt";

    assert_eq!(sensitive_categories(value), vec!["private-path"]);
    assert_eq!(redact_text(value), "[REDACTED_PATH]");
}

#[test]
fn non_phone_parenthesis_before_private_tmp_path_does_not_invert_span() {
    let html = "<p>artifact- (/private/tmp/session/review-artifact.html)</p>";

    assert!(sensitive_categories(html).contains(&"private-path"));
    let redacted = redact_text(html);
    assert!(redacted.contains("[REDACTED_PATH]"));
    assert!(!redacted.contains("/private/tmp/session"));
}

#[test]
fn opaque_and_canonical_secret_tokens_keep_their_specific_categories() {
    assert_eq!(
        sensitive_categories("reference AbCdEfGhIjKlMnOpQrStUvWxYz1234567890"),
        vec!["opaque-token"]
    );
    assert_eq!(
        sensitive_categories("credential AKIAIOSFODNN7EXAMPLE"),
        vec!["secret-token"]
    );
}
