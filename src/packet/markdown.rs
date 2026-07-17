use crate::redaction::render_untrusted_markdown;

fn render_markdown_path(label: &str, path: &std::path::Path) -> String {
    render_untrusted_markdown(label, &path.display().to_string())
}

fn render_optional_markdown_path(label: &str, path: Option<&std::path::Path>) -> String {
    match path {
        Some(path) => render_markdown_path(label, path),
        None => render_untrusted_markdown(label, "not-generated"),
    }
}
