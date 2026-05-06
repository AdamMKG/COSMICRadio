# COSMIC Radio Applet

A minimalist, native COSMIC panel applet for streaming internet radio.

## Features

- **Phase 1**: Basic cosmic_applet scaffold with Hello World popup
- **Phase 2**: Stations.toml config loading with default stations
- **Phase 3**: GStreamer audio playback with playbin
- **Phase 4**: libcosmic widget styling for native look-and-feel

## Usage

1. Click the radio icon in the COSMIC panel
2. Select a station to start playing
3. Use the Play/Stop button to control playback
4. Adjust volume with the slider
5. Click "Edit Stations" to add/remove stations via text editor

## Configuration

Stations are stored in `~/.config/cosmic-radio/stations.toml`:

```toml
[[stations]]
name = "Station Name"
url = "https://stream.url/stream.mp3"
```

## Building

```bash
cargo build
```

## Git Restore Points

- `29af3e2`: Phase 1 complete
- `ac0e6ea`: Phase 2 complete  
- `e757d6f`: Phase 3 + 4 complete
