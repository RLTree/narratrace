# Microphone Input Policy

- `--input auto` prefers the current non-iPhone macOS default microphone when
  AVFoundation exposes it.
- If the default is not selectable by index, auto falls back to the macOS device
  name, then AirPods, then `MacBook Pro Microphone`, then the first non-iPhone
  non-virtual input.
- Never select an iPhone or virtual device for normal narrated capture.
- If auto resolution fails during a live run, inspect:

```sh
system_profiler SPAudioDataType
ffmpeg -hide_banner -f avfoundation -list_devices true -i ""
```

- Retry preparation with an explicit physical input before asking the user to
  re-describe the feature:

```sh
--input ":Terry’s AirPods Pro #3"
--input ":MacBook Pro Microphone"
```

- Record the resolved input from preflight or status artifacts in the dogfood
  receipt. Do not treat a run as narrated if the helper failed before writing
  audio input metadata, retained audio, or transcript artifacts.
