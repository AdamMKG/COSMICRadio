# COSMIC Radio Applet

<img width="50%" height="auto" alt="Screenshot_2026-05-19_11-53-31" src="https://github.com/user-attachments/assets/37fe4baa-4a7e-432d-a98e-748dcd9f30c4" />

A minimalist, native COSMIC panel applet for streaming internet radio stations. Built with Rust for the COSMIC desktop environment.

## Purpose

COSMIC Radio provides a simple, integrated way to stream internet radio stations directly from the COSMIC panel. It features a clean, native interface that follows COSMIC design guidelines, with support for custom station management and album artwork display.

## Tech Stack

### Core Language
- **Rust** (edition 2021) - Systems programming language focused on safety and performance

### Key Dependencies
- **libcosmic** - COSMIC desktop environment toolkit for native applet integration, providing:
  - Panel applet support
  - Wayland integration
  - tokio async runtime
  - Iced-based widget system
- **GStreamer** (0.23) - Multimedia framework for audio playback with:
  - `gstreamer-play` for simplified playback control
  - playbin3 for automatic pipeline management
- **Tokio** (1.x) - Async runtime for non-blocking operations
- **Serde** (1.x) - Serialization/deserialization framework with derive support
- **TOML** (0.8) - Configuration file format parser
- **Reqwest** (0.12) - HTTP client for fetching album artwork
- **SHA2** (0.10) - Cryptographic hashing for artwork caching
- **Dirs** (5.0) - Cross-platform user directory resolution

### Build System
- **Cargo** - Rust package manager and build tool
- **Just** - Command runner for build/install tasks (see `justfile`)

## Features

- Minimalist interface allowing control of volume and playback directly from the tray
- Easy to configure custom stations and groupings through .toml file
- Auto add streams by URL or via JSON API endpoints
- Auto configure highest quality stream from .pls format

## Usage

1. Click the radio icon in the COSMIC panel
2. Select a station to start playing
3. Use the Play/Stop button to control playback
4. Adjust volume with the slider
5. View album artwork when available
6. Click "Edit Stations" to add/remove stations via text editor

## Installation

Install the prebuilt `.deb` from the [latest release](https://github.com/AdamMKG/COSMICRadio/releases):

```bash
curl -LO https://github.com/AdamMKG/COSMICRadio/releases/latest/download/cosmic-radio_0.2.2-1_amd64.deb
sudo apt install ./cosmic-radio_0.2.2-1_amd64.deb
```

`apt` resolves the GStreamer dependency automatically. The release assets also include a standalone `cosmic-radio` binary. Once installed, add the applet to the COSMIC panel by searching for **COSMIC Radio**. It exposes MPRIS2 via `org.mpris.MediaPlayer2.cosmicradio`, so it works with `playerctl` and other media controls.

## Configuration

Stations are stored in `~/.config/cosmic-radio/stations.toml`:

```toml
[[stations]]
name = "Station Name"
url = "https://stream.url/stream.mp3"
artwork = "https://optional-artwork-url.jpg"  # Optional
```

## Building

### Prerequisites
- Rust toolchain (rustc, cargo)
- GStreamer development libraries (1.24+)
- COSMIC desktop environment (for running the applet)

### Build Commands

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release
```

Or use the justfile:
```bash
just build      # Development build
just install    # Install to system
```

## Development

This project was created with the assistance of **Opencode**, an AI-powered coding assistant. Opencode helped with:
- Initial project scaffolding and structure
- Implementing GStreamer integration
- Configuring libcosmic applet components
- Styling with COSMIC widgets
- Album artwork fetching implementation

Human developers retain full responsibility for code review, testing, and functionality decisions.

