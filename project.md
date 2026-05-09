# COSMIC Radio — Project Journal

## Session 1 — 9 May 2026

### Initial State

COSMIC Radio is a Rust applet for the COSMIC desktop that plays internet radio streams
via GStreamer. Prior to this session, the app was hardcoded to work specifically with
SomaFM stations — it had a `derive_artwork_url()` function that only knew how to
derive artwork URLs from `somafm.com` stream URLs, and the fallback default config
shipped with a hardcoded SomaFM station entry.

The app supported:
- TOML-based station config with grouped stations
- Collapsible station groups
- Artwork caching (SHA256-hashed URLs, 5-day TTL)
- Volume slider with GStreamer playback
- Opening the TOML file in the system editor (VISUAL/EDITOR env, cosmic-edit, xdg-open)

### Changes Made This Session

#### General-purpose station support
- Removed `derive_artwork_url()` — artwork is no longer auto-derived from stream URLs.
  Artwork must be explicitly specified in the station config.
- Simplified `load_artwork()` — returns early if a station has no explicit artwork URL.
- Removed the SomaFM hardcoded default from `ensure_config()`. The fallback now
  only copies from `/usr/share/cosmic-radio/stations.toml` if available; otherwise
  creates an empty config directory.

#### PLS playlist support
- Added `resolve_pls(content) -> Option<(String, String)>` — parses the standard
  SHOUTcast/Icecast PLS `[playlist]` format, extracting `File1` (stream URL) and
  `Title1` (station name).
- Added `resolve_pls_url(url)` — fetches PLS content from HTTP(S) URLs or local file
  paths, then parses it.
- Modified `start_playback()` — transparently detects `.pls` URLs and resolves them
  to the actual stream URL and display name before playback.
- Added `blocking` feature to the `reqwest` dependency for synchronous PLS fetching.

#### Now Playing metadata
- Added GStreamer bus sync handler that listens for `Tag` messages on the pipeline bus.
- Extracts `Artist` and `Title` tags from stream metadata and formats them as
  `"Artist - Title"`.
- Metadata is stored in an `Arc<Mutex<Option<String>>>` for thread-safe sharing
  between the GStreamer streaming thread and the iced event loop.
- The `now_playing` field shows stream metadata when available, falling back to the
  station name. Resets on station change.

#### Scrolling marquee
- Added a `MarqueeTick` message driven by a 50ms timer subscription.
- When now-playing text exceeds 29 characters (approximately 220px), it scrolls
  horizontally through the content using character-based substring rotation.
- Scrolling pauses briefly at the end before resetting.
- The marquee is rendered inside a clipped `container` (fixed 220px width, `clip(true)`).
- Scroll offset resets when a new station is selected or new metadata arrives.

### Files Changed
- `Cargo.toml` — added `blocking` feature to `reqwest`
- `src/main.rs` — main implementation changes (see above)
- `Cargo.lock` — updated via `cargo build`

### Current Architecture

```
src/main.rs (599 lines)
├── Data types: Station, StationGroup, Config, OldConfig
├── Config helpers: config_path(), load_config(), ensure_config()
├── PLS parser: resolve_pls(), resolve_pls_url()
├── RadioApp struct (core app state)
├── Message enum (9 variants)
├── Application impl (init, update, view, subscription, style)
├── RadioApp impl (playback, artwork, cache)
└── main()
```
