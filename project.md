# COSMIC Radio — Project Journal

> **Note:** Update this journal as significant changes are made to the project. Log each session with the date, what changed, and why.

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

## Session 2 — 9 May 2026

### Changes Made This Session

#### "+" button with context menu (replaces "Edit Stations" button)
- Removed the `EditStations` message variant — replaced with `ToggleAddMenu`, `AddCurrentlyPlaying`, `AddViaUrl`, and `EditStationsToml`.
- Added `show_add_menu: bool` field to `RadioApp` to track whether the add context menu is open.
- The bottom bar now shows a `"+"` button instead of "Edit Stations". Tapping it toggles a context menu with three options: "Add currently playing", "Add via URL", and "Edit stations.toml".
- All three menu items currently close the menu on selection; `EditStationsToml` opens the config file in the system editor (same logic as the old `EditStations`). The other two are placeholders awaiting logic.

#### Add currently playing logic
- Added `auto_add: Option<bool>` field (TOML key `auto-add`) to `Station` with `#[serde(skip_serializing_if = "Option::is_none")]` so it only writes when `true`.
- Added `Serialize` derives to `Station`, `StationGroup`, and `Config`.
- `AddCurrentlyPlaying` now reads the config from disk, checks if the current station's name or URL already exists in any group, and if neither does, appends it to a "Favourites" group (creating it if needed) with `auto_add = true`.

### Files Changed
- `src/main.rs` — struct field, message enum, update handlers, view layout, serialize derives

## Session 3 — 15 May 2026

### Changes Made This Session

#### Project restructured into modules
- Split monolithic `src/main.rs` (665 lines) into separate modules:
  - `src/config.rs` — `ConfigManager`, `Station`, `StationGroup`, TOML load/save, backward compat with `OldConfig`
  - `src/audio.rs` — `AudioBackend` wrapping GStreamer `Play`, PLS resolution, metadata bus handler
  - `src/artwork.rs` — `ArtworkCache` with async download, SHA256 caching, 5-day TTL
  - `src/app.rs` — `RadioApp`, `Message` enum, COSMIC `Application` impl (update, view, subscription)
  - `src/main.rs` — reduced to entry point only (3 lines)
- `ConfigManager` caches a flat station list internally, recalculated on `add_station()`
- `AudioBackend::take_metadata()` replaces the raw `Arc<Mutex<...>>` in `RadioApp`
- `ArtworkCache` owns the `HashMap<usize, PathBuf>` and handles all cache logic
- RadioApp's `volume` field is mirrored to `AudioBackend` on `set_volume()` — no volume state leak
- "Add currently playing" now calls `ConfigManager::add_station()` which immediately updates in-memory groups/flat list and syncs to disk

### Files Changed
- `src/main.rs` — gutted to entry point only, declares submodules
- `src/config.rs` — new file (ConfigManager, all config types and I/O)
- `src/audio.rs` — new file (AudioBackend, GStreamer, PLS, metadata)
- `src/artwork.rs` — new file (ArtworkCache, download, cache)
- `src/app.rs` — new file (RadioApp, Message, cosmic::Application impl)

## Session 3b — 15 May 2026

### Changes Made This Session

#### Play/Stop icon button (replaces text button)
- Replaced the text-based "Play"/"Stop" button with SVG icons embedded at compile time via `include_bytes!("../data/play_button.svg")` and `include_bytes!("../data/stop_button.svg")`.
- Icons are rendered using `iced::widget::svg` at 24×24px inside a `button::custom()` with 4px padding.
- Added a `tooltip::Tooltip` wrapper so hovering over the button shows "Play" or "Stop" for accessibility.
- Button logic: if no station is selected, the play icon is always shown (never switches to stop). If a station is selected, the icon toggles between play (paused) and stop (playing).

#### Layout reorganisation
- Moved the play/stop button from its own dedicated row (below the now-playing row) into the now-playing row itself, right-aligned.
- The now-playing row layout is now `[artwork | marquee text (Fill) | play/stop button]` — the marquee text uses `iced::Length::Fill` so it expands to fill the remaining horizontal space between the artwork and the button.
- This removed the separate play button row from the content column, reducing vertical space usage while keeping the same overall popup dimensions.
- The marquee text now has less horizontal space available, which is acceptable to maintain the overall popup width.

### Files Changed
- `src/app.rs` — imports (svg, tooltip), constants (PLAY_SVG, STOP_SVG), now_playing_row layout, play button icon + tooltip, content column

### Current Architecture
```
src/
├── main.rs      (3 lines)  — cosmic::applet::run::<RadioApp>
├── config.rs    (137 lines) — ConfigManager, types, TOML I/O
├── audio.rs     (100 lines) — AudioBackend, GStreamer, PLS, metadata
├── artwork.rs   (89 lines)  — ArtworkCache, async download, hashing
└── app.rs       (440 lines) — RadioApp, Message, UI, cosmic::Application
```
