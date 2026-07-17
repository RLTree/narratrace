fn read_json(path: &Path) -> Result<Value> {
    let text = crate::safe_path::read_regular_text_bounded(path, MAX_REVIEW_JSON_BYTES)?;
    Ok(serde_json::from_str(&text)?)
}

fn capture_audio_input_label(receipt: &Value) -> String {
    let input = receipt
        .pointer("/capture/audioInput")
        .unwrap_or(&Value::Null);
    let device = input
        .get("deviceName")
        .and_then(Value::as_str)
        .unwrap_or("unknown-device");
    let ffmpeg_input = input
        .get("ffmpegInput")
        .and_then(Value::as_str)
        .unwrap_or("unknown-input");
    let source = input
        .get("source")
        .and_then(Value::as_str)
        .unwrap_or("unknown-source");
    format!("{device} ({ffmpeg_input}, {source})")
}

fn regular_file_exists(path: &Path) -> bool {
    regular_file_metadata(path).is_ok()
}
