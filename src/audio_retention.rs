use crate::config::SAMPLE_RATE;
use crate::private_fs::{create_private_dir_all, create_private_file, write_private};
use anyhow::Result;
use serde_json::json;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

const CHANNELS: u16 = 1;
const BITS_PER_SAMPLE: u16 = 16;
const BYTES_PER_SAMPLE: u32 = (BITS_PER_SAMPLE as u32) / 8;

pub struct AudioRetentionWriter {
    path: PathBuf,
    file: File,
    data_bytes: u32,
    chunks: Vec<RetainedAudioChunk>,
}

#[derive(Debug, Clone)]
pub struct RetainedAudioChunk {
    pub sample_start: u64,
    pub sample_end: u64,
    pub byte_start: u64,
    pub byte_end: u64,
    pub monotonic_offset_ms: u64,
}

impl AudioRetentionWriter {
    pub fn create(
        session_dir: &Path,
        explicit_path: Option<&Path>,
        mode: &str,
    ) -> Result<Option<Self>> {
        if mode == "disabled" {
            write_retention_manifest(session_dir, None, mode, 0)?;
            return Ok(None);
        }
        let path = explicit_path
            .map(Path::to_path_buf)
            .unwrap_or_else(|| session_dir.join("retained-audio.wav"));
        if let Some(parent) = path.parent() {
            create_private_dir_all(parent)?;
        }
        let mut file = create_private_file(&path)?;
        write_wav_header(&mut file, 0)?;
        write_retention_manifest(session_dir, Some(&path), mode, 0)?;
        Ok(Some(Self {
            path,
            file,
            data_bytes: 0,
            chunks: Vec::new(),
        }))
    }

    pub fn append(&mut self, audio: &[u8], monotonic_offset_ms: u64) -> Result<()> {
        let byte_start = self.data_bytes as u64;
        self.file.write_all(audio)?;
        self.data_bytes = self
            .data_bytes
            .saturating_add(u32::try_from(audio.len()).unwrap_or(u32::MAX));
        let byte_end = self.data_bytes as u64;
        self.chunks.push(RetainedAudioChunk {
            sample_start: byte_start / BYTES_PER_SAMPLE as u64,
            sample_end: byte_end / BYTES_PER_SAMPLE as u64,
            byte_start,
            byte_end,
            monotonic_offset_ms,
        });
        Ok(())
    }

    pub fn finalize(mut self, session_dir: &Path, mode: &str) -> Result<PathBuf> {
        self.file.seek(SeekFrom::Start(0))?;
        write_wav_header(&mut self.file, self.data_bytes)?;
        self.file.flush()?;
        write_retention_manifest(session_dir, Some(&self.path), mode, self.data_bytes)?;
        write_chunk_manifest(session_dir, &self.chunks)?;
        Ok(self.path)
    }
}

fn write_wav_header(file: &mut File, data_bytes: u32) -> Result<()> {
    let byte_rate = SAMPLE_RATE * CHANNELS as u32 * BYTES_PER_SAMPLE;
    let block_align = CHANNELS * (BITS_PER_SAMPLE / 8);
    let riff_size = 36_u32.saturating_add(data_bytes);
    file.write_all(b"RIFF")?;
    file.write_all(&riff_size.to_le_bytes())?;
    file.write_all(b"WAVEfmt ")?;
    file.write_all(&16_u32.to_le_bytes())?;
    file.write_all(&1_u16.to_le_bytes())?;
    file.write_all(&CHANNELS.to_le_bytes())?;
    file.write_all(&SAMPLE_RATE.to_le_bytes())?;
    file.write_all(&byte_rate.to_le_bytes())?;
    file.write_all(&block_align.to_le_bytes())?;
    file.write_all(&BITS_PER_SAMPLE.to_le_bytes())?;
    file.write_all(b"data")?;
    file.write_all(&data_bytes.to_le_bytes())?;
    Ok(())
}

fn write_retention_manifest(
    session_dir: &Path,
    path: Option<&Path>,
    mode: &str,
    data_bytes: u32,
) -> Result<()> {
    write_private(
        session_dir.join("audio-retention.json"),
        serde_json::to_string_pretty(&json!({
            "schema": "narrated-record-replay.audio-retention.v1",
            "mode": mode,
            "audioPath": path.map(|path| path.display().to_string()),
            "format": if path.is_some() { "wav/pcm_s16le" } else { "not-retained" },
            "sampleRate": SAMPLE_RATE,
            "channels": CHANNELS,
            "dataBytes": data_bytes,
            "samples": data_bytes / BYTES_PER_SAMPLE,
            "privacy": {
                "localPrivate": true,
                "copyIntoGeneratedPacketsByDefault": false,
                "containsRawMicrophoneAudio": path.is_some(),
                "retentionNote": "This file is the exact filtered mono PCM stream sent to realtime transcription, wrapped as WAV for post-run transcription. Keep it under private runtime paths and do not copy it into repo artifacts or shared outputs by default."
            }
        }))?,
    )
}

fn write_chunk_manifest(session_dir: &Path, chunks: &[RetainedAudioChunk]) -> Result<()> {
    write_private(
        session_dir.join("audio-chunks.jsonl"),
        chunks
            .iter()
            .enumerate()
            .map(|(index, chunk)| {
                serde_json::to_string(&json!({
                    "schema": "narrated-record-replay.audio-chunk.v1",
                    "index": index + 1,
                    "sampleStart": chunk.sample_start,
                    "sampleEnd": chunk.sample_end,
                    "byteStart": chunk.byte_start,
                    "byteEnd": chunk.byte_end,
                    "monotonicOffsetMs": chunk.monotonic_offset_ms,
                    "sampleRate": SAMPLE_RATE,
                    "privacy": {
                        "rawAudioCopied": false,
                        "metadataOnly": true
                    }
                }))
            })
            .collect::<Result<Vec<_>, _>>()?
            .join("\n")
            + "\n",
    )
}

#[cfg(test)]
#[path = "audio_retention_tests.rs"]
mod tests;
