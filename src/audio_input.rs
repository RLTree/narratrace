use anyhow::{Context, Result, bail};
use std::process::Command;

const SYSTEM_PROFILER_PATH: &str = "/usr/sbin/system_profiler";

mod devices;

use devices::{
    is_iphone_device, is_valid_avfoundation_device_name, is_virtual_device,
    parse_avfoundation_audio_devices, parse_default_input_name, parse_system_input_device_names,
    select_preferred_audio_device, select_preferred_audio_device_name,
};

pub const AUTO_INPUT: &str = "auto";
pub const AUTO_INPUT_POLICY: &str = "auto uses the current non-iPhone macOS default input when ffmpeg exposes it, then falls back to the macOS device name, then AirPods, then MacBook Pro Microphone, then the first non-iPhone non-virtual input";

#[derive(Debug, Clone)]
pub struct ResolvedAudioInput {
    pub requested: String,
    pub ffmpeg_input: String,
    pub device_name: Option<String>,
    pub source: String,
}

pub fn resolve_avfoundation_input(requested: &str) -> Result<ResolvedAudioInput> {
    let devices_text =
        if is_auto_input(requested) || parse_requested_audio_index(requested).is_some() {
            Some(avfoundation_devices_text()?)
        } else {
            None
        };
    let system_audio_text = if is_auto_input(requested) {
        system_audio_devices_text().ok()
    } else {
        None
    };
    resolve_avfoundation_input_from_texts(
        requested,
        devices_text.as_deref(),
        system_audio_text.as_deref(),
    )
}

fn resolve_avfoundation_input_from_texts(
    requested: &str,
    devices_text: Option<&str>,
    system_audio_text: Option<&str>,
) -> Result<ResolvedAudioInput> {
    if !is_auto_input(requested) {
        if let Some(index) = parse_requested_audio_index(requested) {
            let devices = parse_avfoundation_audio_devices(devices_text.unwrap_or_default());
            let device = devices
                .iter()
                .find(|device| device.index == index)
                .ok_or_else(|| {
                    anyhow::anyhow!("AVFoundation audio input index {index} was not found")
                })?;
            return Ok(ResolvedAudioInput {
                requested: requested.to_string(),
                ffmpeg_input: requested.to_string(),
                device_name: Some(device.name.clone()),
                source: "explicit-avfoundation-index".to_string(),
            });
        }
        if let Some(name) = parse_requested_audio_name(requested) {
            return Ok(ResolvedAudioInput {
                requested: requested.to_string(),
                ffmpeg_input: requested.to_string(),
                device_name: Some(name),
                source: "explicit-avfoundation-name".to_string(),
            });
        }
        bail!(
            "AVFoundation input must be auto, a resolved audio index like :4, or a device name like ':MacBook Pro Microphone'"
        );
    }

    let devices = parse_avfoundation_audio_devices(devices_text.unwrap_or_default());
    let default_name = system_audio_text.and_then(parse_default_input_name);
    let selected = select_preferred_audio_device(&devices, default_name.as_deref());

    if let Some(selected) = selected {
        return Ok(ResolvedAudioInput {
            requested: requested.to_string(),
            ffmpeg_input: format!(":{}", selected.index),
            device_name: Some(selected.name.clone()),
            source: selected.source,
        });
    }

    let system_inputs = system_audio_text
        .map(parse_system_input_device_names)
        .unwrap_or_default();
    let selected_name = select_preferred_audio_device_name(&system_inputs, default_name.as_deref())
        .ok_or_else(|| anyhow::anyhow!("no suitable non-iPhone AVFoundation audio input found"))?;

    Ok(ResolvedAudioInput {
        requested: requested.to_string(),
        ffmpeg_input: format!(":{}", selected_name.name),
        device_name: Some(selected_name.name.clone()),
        source: selected_name.source,
    })
}

fn is_auto_input(input: &str) -> bool {
    matches!(input, "" | AUTO_INPUT | ":default" | "default")
}

fn parse_requested_audio_index(input: &str) -> Option<u32> {
    let value = input.strip_prefix(':')?;
    if value.is_empty() || value.contains(':') {
        return None;
    }
    value.parse::<u32>().ok()
}

fn parse_requested_audio_name(input: &str) -> Option<String> {
    let value = input.strip_prefix(':')?.trim();
    if !is_valid_avfoundation_device_name(value) || value.chars().all(|ch| ch.is_ascii_digit()) {
        return None;
    }
    Some(value.to_string())
}

fn avfoundation_devices_text() -> Result<String> {
    let output = Command::new(crate::realtime::ffmpeg_binary()?)
        .args([
            "-hide_banner",
            "-f",
            "avfoundation",
            "-list_devices",
            "true",
            "-i",
            "",
        ])
        .output()
        .context("failed to list AVFoundation devices with ffmpeg")?;
    let mut text = String::from_utf8_lossy(&output.stderr).to_string();
    text.push_str(&String::from_utf8_lossy(&output.stdout));
    Ok(text)
}

fn system_audio_devices_text() -> Result<String> {
    let output = Command::new(SYSTEM_PROFILER_PATH)
        .arg("SPAudioDataType")
        .output()
        .context("failed to inspect macOS audio devices")?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

pub fn ensure_not_iphone_input(input: &ResolvedAudioInput) -> Result<()> {
    if let Some(device_name) = input.device_name.as_deref() {
        if is_iphone_device(device_name) {
            bail!("refusing to use iPhone microphone as narrated replay input");
        }
        if is_virtual_device(device_name) {
            bail!("refusing to use virtual audio device as narrated replay input");
        }
    }
    Ok(())
}

pub fn auto_input_preview() -> serde_json::Value {
    match resolve_avfoundation_input(AUTO_INPUT) {
        Ok(input) => {
            let rejected = ensure_not_iphone_input(&input).is_err();
            serde_json::json!({
                "status": if rejected { "rejected" } else { "resolved" },
                "requested": input.requested,
                "ffmpegInput": input.ffmpeg_input,
                "deviceName": input.device_name,
                "source": input.source,
                "policy": AUTO_INPUT_POLICY,
                "rejectsIphoneOrVirtualInput": rejected
            })
        }
        Err(error) => serde_json::json!({
            "status": "unresolved",
            "error": error.to_string(),
            "policy": AUTO_INPUT_POLICY
        }),
    }
}

#[cfg(test)]
mod tests;
