# COSMIC Radio Applet

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

- **Phase 1**: Basic cosmic_applet scaffold with Hello World popup
- **Phase 2**: Stations.toml config loading with default stations
- **Phase 3**: GStreamer audio playback with playbin
- **Phase 4**: libcosmic widget styling for native look-and-feel
- **Current**: Album artwork fetching and display from station URLs

## Usage

1. Click the radio icon in the COSMIC panel
2. Select a station to start playing
3. Use the Play/Stop button to control playback
4. Adjust volume with the slider
5. View album artwork when available
6. Click "Edit Stations" to add/remove stations via text editor

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

## Git Restore Points

- `29af3e2`: Phase 1 complete
- `ac0e6ea`: Phase 2 complete  
- `e757d6f`: Phase 3 + 4 complete
- `963ab48`: README and Phase 4 styling complete
- `991fcdb`: .desktop file and install configuration
- `e4b4495`: justfile for build/install commands
