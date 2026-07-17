#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AudioDevice {
    pub(super) index: u32,
    pub(super) name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SelectedAudioDevice {
    pub(super) index: u32,
    pub(super) name: String,
    pub(super) source: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SelectedAudioDeviceName {
    pub(super) name: String,
    pub(super) source: String,
}

pub(super) fn parse_avfoundation_audio_devices(text: &str) -> Vec<AudioDevice> {
    let mut in_audio = false;
    let mut devices = Vec::new();
    for line in text.lines() {
        if line.contains("AVFoundation audio devices:") {
            in_audio = true;
            continue;
        }
        if line.contains("AVFoundation video devices:") {
            in_audio = false;
            continue;
        }
        if !in_audio {
            continue;
        }
        if let Some((index, name)) = parse_indexed_device(line) {
            devices.push(AudioDevice { index, name });
        }
    }
    devices
}

pub(super) fn parse_indexed_device(line: &str) -> Option<(u32, String)> {
    let open = line.rfind('[')?;
    let close = line[open + 1..].find(']')? + open + 1;
    let index = line[open + 1..close].trim().parse::<u32>().ok()?;
    let name = line[close + 1..].trim();
    if name.is_empty() {
        None
    } else {
        Some((index, name.to_string()))
    }
}

pub(super) fn parse_default_input_name(text: &str) -> Option<String> {
    let mut current: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.ends_with(':') && !trimmed.contains(": ") {
            current = Some(trimmed.trim_end_matches(':').to_string());
            continue;
        }
        if trimmed == "Default Input Device: Yes" {
            return current.clone();
        }
    }
    None
}

pub(super) fn parse_system_input_device_names(text: &str) -> Vec<String> {
    let mut current: Option<String> = None;
    let mut current_has_input = false;
    let mut devices = Vec::new();

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.ends_with(':') && !trimmed.contains(": ") {
            push_system_input_device(&mut devices, current.take(), current_has_input);
            current = Some(trimmed.trim_end_matches(':').to_string());
            current_has_input = false;
            continue;
        }
        if trimmed.starts_with("Input Channels:") {
            current_has_input = true;
        }
    }
    push_system_input_device(&mut devices, current, current_has_input);
    devices
}

fn push_system_input_device(devices: &mut Vec<String>, current: Option<String>, has_input: bool) {
    if has_input
        && let Some(name) = current
        && is_valid_avfoundation_device_name(&name)
    {
        devices.push(name);
    }
}

pub(super) fn select_preferred_audio_device(
    devices: &[AudioDevice],
    default_name: Option<&str>,
) -> Option<SelectedAudioDevice> {
    if let Some(default_name) = default_name
        && !is_iphone_device(default_name)
        && let Some(device) = find_device(devices, default_name)
    {
        return Some(selected(device, "macos-default-input"));
    }
    if let Some(device) = devices
        .iter()
        .find(|device| is_airpods_device(&device.name))
    {
        return Some(selected(device, "airpods-fallback"));
    }
    if let Some(device) = devices
        .iter()
        .find(|device| device.name == "MacBook Pro Microphone")
    {
        return Some(selected(device, "macbook-microphone-fallback"));
    }
    devices
        .iter()
        .find(|device| !is_iphone_device(&device.name) && !is_virtual_device(&device.name))
        .map(|device| selected(device, "first-non-iphone-input"))
}

pub(super) fn select_preferred_audio_device_name(
    names: &[String],
    default_name: Option<&str>,
) -> Option<SelectedAudioDeviceName> {
    if let Some(default_name) = default_name
        && is_selectable_physical_input(default_name)
        && names.iter().any(|name| name == default_name)
    {
        return Some(selected_name(
            default_name,
            "macos-default-input-name-fallback",
        ));
    }
    if let Some(name) = names.iter().find(|name| is_airpods_device(name)) {
        return Some(selected_name(name, "airpods-name-fallback"));
    }
    if let Some(name) = names
        .iter()
        .find(|name| name.as_str() == "MacBook Pro Microphone")
    {
        return Some(selected_name(name, "macbook-microphone-name-fallback"));
    }
    names
        .iter()
        .find(|name| is_selectable_physical_input(name))
        .map(|name| selected_name(name, "first-non-iphone-input-name-fallback"))
}

fn find_device<'a>(devices: &'a [AudioDevice], name: &str) -> Option<&'a AudioDevice> {
    devices.iter().find(|device| device.name == name)
}

fn selected(device: &AudioDevice, source: &str) -> SelectedAudioDevice {
    SelectedAudioDevice {
        index: device.index,
        name: device.name.clone(),
        source: source.to_string(),
    }
}

fn selected_name(name: &str, source: &str) -> SelectedAudioDeviceName {
    SelectedAudioDeviceName {
        name: name.to_string(),
        source: source.to_string(),
    }
}

fn is_selectable_physical_input(name: &str) -> bool {
    is_valid_avfoundation_device_name(name) && !is_iphone_device(name) && !is_virtual_device(name)
}

pub(super) fn is_valid_avfoundation_device_name(name: &str) -> bool {
    let trimmed = name.trim();
    !trimmed.is_empty()
        && !trimmed.contains(':')
        && !trimmed.contains('\0')
        && !trimmed.contains('\n')
        && !trimmed.contains('\r')
}

pub(super) fn is_iphone_device(name: &str) -> bool {
    name.to_ascii_lowercase().contains("iphone")
}

pub(super) fn is_airpods_device(name: &str) -> bool {
    name.to_ascii_lowercase().contains("airpods")
}

pub(super) fn is_virtual_device(name: &str) -> bool {
    let lowered = name.to_ascii_lowercase();
    lowered.contains("zoom")
        || lowered.contains("teams")
        || lowered.contains("loopback")
        || lowered.contains("blackhole")
        || lowered.contains("soundflower")
        || lowered.contains("aggregate")
        || lowered.contains("multi-output")
        || lowered.contains("virtual")
}
