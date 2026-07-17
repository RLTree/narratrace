use super::*;

const AVFOUNDATION_DEVICES: &str = r#"
[AVFoundation indev @ 0x1] AVFoundation video devices:
[AVFoundation indev @ 0x1] [0] MacBook Pro Camera
[AVFoundation indev @ 0x1] AVFoundation audio devices:
[AVFoundation indev @ 0x1] [0] Terry’s iPhone Microphone
[AVFoundation indev @ 0x1] [1] MacBook Pro Microphone
[AVFoundation indev @ 0x1] [2] ZoomAudioDevice
[AVFoundation indev @ 0x1] [3] Microsoft Teams Audio
[AVFoundation indev @ 0x1] [4] Terry’s AirPods Pro #3
"#;

const SYSTEM_AUDIO: &str = r#"
Audio:

    Devices:

        Terry’s iPhone Microphone:

          Input Channels: 1

        Terry’s AirPods Pro #3:

          Default Input Device: Yes
          Input Channels: 1

        MacBook Pro Microphone:

          Input Channels: 1

        MacBook Pro Speakers:

          Output Channels: 2
"#;

#[test]
fn parses_avfoundation_audio_devices() {
    let devices = parse_avfoundation_audio_devices(AVFOUNDATION_DEVICES);
    assert_eq!(devices[0].name, "Terry’s iPhone Microphone");
    assert_eq!(devices[4].index, 4);
    assert_eq!(devices[4].name, "Terry’s AirPods Pro #3");
}

#[test]
fn parses_macos_default_input() {
    assert_eq!(
        parse_default_input_name(SYSTEM_AUDIO).as_deref(),
        Some("Terry’s AirPods Pro #3")
    );
}

#[test]
fn parses_macos_input_device_names() {
    let names = parse_system_input_device_names(SYSTEM_AUDIO);

    assert_eq!(
        names,
        vec![
            "Terry’s iPhone Microphone".to_string(),
            "Terry’s AirPods Pro #3".to_string(),
            "MacBook Pro Microphone".to_string()
        ]
    );
}

#[test]
fn prefers_non_iphone_default_input() {
    let devices = parse_avfoundation_audio_devices(AVFOUNDATION_DEVICES);
    let selected = select_preferred_audio_device(&devices, Some("Terry’s AirPods Pro #3")).unwrap();
    assert_eq!(selected.index, 4);
    assert_eq!(selected.source, "macos-default-input");
}

#[test]
fn skips_iphone_default_for_airpods_then_macbook() {
    let devices = parse_avfoundation_audio_devices(AVFOUNDATION_DEVICES);
    let selected =
        select_preferred_audio_device(&devices, Some("Terry’s iPhone Microphone")).unwrap();
    assert_eq!(selected.name, "Terry’s AirPods Pro #3");
    assert_eq!(selected.source, "airpods-fallback");
}

#[test]
fn falls_back_to_system_default_name_when_avfoundation_listing_is_empty() {
    let names = parse_system_input_device_names(SYSTEM_AUDIO);
    let selected =
        select_preferred_audio_device_name(&names, Some("Terry’s AirPods Pro #3")).unwrap();

    assert_eq!(selected.name, "Terry’s AirPods Pro #3");
    assert_eq!(selected.source, "macos-default-input-name-fallback");
}

#[test]
fn skips_iphone_system_default_for_airpods_name_fallback() {
    let names = parse_system_input_device_names(SYSTEM_AUDIO);
    let selected =
        select_preferred_audio_device_name(&names, Some("Terry’s iPhone Microphone")).unwrap();

    assert_eq!(selected.name, "Terry’s AirPods Pro #3");
    assert_eq!(selected.source, "airpods-name-fallback");
}

#[test]
fn allows_explicit_non_numeric_avfoundation_audio_name() {
    assert_eq!(
        parse_requested_audio_name(":MacBook Pro Microphone").as_deref(),
        Some("MacBook Pro Microphone")
    );
    assert_eq!(
        parse_requested_audio_name(":Terry’s AirPods Pro #3").as_deref(),
        Some("Terry’s AirPods Pro #3")
    );
    assert_eq!(parse_requested_audio_name(":4"), None);
    assert_eq!(parse_requested_audio_name(":Bad:Name"), None);
}

#[test]
fn parses_explicit_audio_index() {
    assert_eq!(parse_requested_audio_index(":4"), Some(4));
    assert_eq!(parse_requested_audio_index(":0"), Some(0));
    assert_eq!(parse_requested_audio_index("0"), None);
    assert_eq!(parse_requested_audio_index("0:1"), None);
}

#[test]
fn auto_input_aliases_are_recognized() {
    assert!(is_auto_input(""));
    assert!(is_auto_input("auto"));
    assert!(is_auto_input("default"));
    assert!(is_auto_input(":default"));
    assert!(!is_auto_input(":MacBook Pro Microphone"));
}

#[test]
fn explicit_input_rejects_unparseable_shapes_before_shelling_out() {
    let error = resolve_avfoundation_input("MacBook Pro Microphone")
        .unwrap_err()
        .to_string();

    assert!(error.contains("AVFoundation input must be auto"));
}

#[test]
fn resolves_explicit_index_from_parsed_avfoundation_text() {
    let resolved =
        resolve_avfoundation_input_from_texts(":4", Some(AVFOUNDATION_DEVICES), None).unwrap();

    assert_eq!(resolved.ffmpeg_input, ":4");
    assert_eq!(
        resolved.device_name.as_deref(),
        Some("Terry’s AirPods Pro #3")
    );
    assert_eq!(resolved.source, "explicit-avfoundation-index");
}

#[test]
fn resolves_auto_from_parsed_device_text_without_shelling_out() {
    let resolved = resolve_avfoundation_input_from_texts(
        "auto",
        Some(AVFOUNDATION_DEVICES),
        Some(SYSTEM_AUDIO),
    )
    .unwrap();

    assert_eq!(resolved.ffmpeg_input, ":4");
    assert_eq!(
        resolved.device_name.as_deref(),
        Some("Terry’s AirPods Pro #3")
    );
    assert_eq!(resolved.source, "macos-default-input");
}

#[test]
fn resolves_auto_from_system_name_when_avfoundation_has_no_audio_devices() {
    let resolved =
        resolve_avfoundation_input_from_texts("auto", Some(""), Some(SYSTEM_AUDIO)).unwrap();

    assert_eq!(resolved.ffmpeg_input, ":Terry’s AirPods Pro #3");
    assert_eq!(
        resolved.device_name.as_deref(),
        Some("Terry’s AirPods Pro #3")
    );
    assert_eq!(resolved.source, "macos-default-input-name-fallback");
}

#[test]
fn auto_resolution_errors_when_only_rejected_devices_are_available() {
    let error = resolve_avfoundation_input_from_texts(
        "auto",
        Some("[AVFoundation indev @ 0x1] AVFoundation audio devices:\n[0] Terry’s iPhone Microphone"),
        Some("Audio:\n\n    Devices:\n\n        Terry’s iPhone Microphone:\n          Default Input Device: Yes\n          Input Channels: 1"),
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("no suitable non-iPhone"));
}

#[test]
fn rejects_resolved_iphone_and_virtual_devices() {
    let iphone = ResolvedAudioInput {
        requested: ":0".to_string(),
        ffmpeg_input: ":0".to_string(),
        device_name: Some("Terry’s iPhone Microphone".to_string()),
        source: "explicit-avfoundation-index".to_string(),
    };
    assert!(ensure_not_iphone_input(&iphone).is_err());

    let virtual_device = ResolvedAudioInput {
        requested: ":2".to_string(),
        ffmpeg_input: ":2".to_string(),
        device_name: Some("ZoomAudioDevice".to_string()),
        source: "explicit-avfoundation-index".to_string(),
    };
    assert!(ensure_not_iphone_input(&virtual_device).is_err());

    let macbook = ResolvedAudioInput {
        requested: ":MacBook Pro Microphone".to_string(),
        ffmpeg_input: ":MacBook Pro Microphone".to_string(),
        device_name: Some("MacBook Pro Microphone".to_string()),
        source: "explicit-avfoundation-name".to_string(),
    };
    assert!(ensure_not_iphone_input(&macbook).is_ok());
}

#[test]
fn system_profiler_uses_the_trusted_absolute_path() {
    let path = std::path::Path::new(SYSTEM_PROFILER_PATH);

    assert!(path.is_absolute());
    assert_eq!(path, std::path::Path::new("/usr/sbin/system_profiler"));
}
