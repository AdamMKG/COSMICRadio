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

## Session 4 — 16 May 2026

### Changes Made This Session

#### Marquee scrolling improvements
- Added `MARQUEE_SCROLL_INTERVAL = 3` — scroll advances every 3rd tick (150ms/char, ~3× slower)
- Added `MARQUEE_START_PAUSE_TICKS = 60.0` — 3-second pause before scrolling begins
- Increased `MARQUEE_END_PAUSE_TICKS` from 20 to 60.0 — 3-second pause at end of scroll (3× longer)
- Added `scroll_tick_counter: u32` to track tick position in the pause→scroll→pause cycle
- Split marquee cycle into three phases: start pause, scrolling, end pause
- Increased `MARQUEE_MAX_CHARS` from 29 to 33 — shows 4 more characters before truncation

#### Layout and button fixes
- Reverted text container width from `Fixed(220.0)` back to `Fill` so the play/stop button stays right-aligned regardless of artwork presence or text length
- Added 4px spacer after the play/stop button so its right edge aligns with the station list scrollbar (total 12px gap from button to popup right edge)
- Replaced "+" unicode character with `add_station.svg` SVG icon, matching the style of the play/stop buttons

#### Play from URL feature
- Renamed "Add via URL" → "Play from URL" in the add menu
- Added `show_url_input`, `url_input`, `temp_stream_url`, `temp_stream_name` state fields
- Added `UrlInputChanged(String)` and `SubmitUrl` message variants
- When "Play from URL" is clicked, a text input and "Play" button appear; the user pastes a URL and the stream plays immediately
- `SubmitUrl` auto-prepends `https://` if no protocol is present in the pasted URL
- PLS URLs are resolved transparently via the existing `AudioBackend::play()` path
- `TogglePlayback` now resumes URL-played streams (not just config-station streams)
- `SelectStation` clears temp stream state

#### Add currently playing — now supports URL streams
- "Add currently playing" now handles both config-station streams (`current_station`) and URL-played streams (`temp_stream_url`)
- Stations are added to an "Uncategorised" group (created if absent) instead of "Favourites"
- `ConfigManager::add_station()` renamed to `add_to_group(station, group_name)` with a configurable group name
- Station names for URL streams are derived from the URL path/hostname via `derive_name_from_url()`, or from the PLS `Title1` field when available

#### Placeholder artwork
- Added `ARTWORK_PLACEHOLDER` constant embedding `station_artwork_placeholder.png`
- `artwork_image()` now renders the placeholder PNG when no artwork path is provided or the cache file is missing, instead of an empty `Space`
- The now-playing row always shows artwork (real or placeholder), keeping the layout consistent

#### Play/Stop button for URL streams
- Fixed the play/stop icon logic: now checks `temp_stream_url` in addition to `current_station`, so the button correctly toggles between Play and Stop for URL-played streams

#### URL handler module (uncommitted until Session 5)
- Extracted URL resolution logic (SomaFM JSON, Radio Browser JSON, PLS, raw audio URLs) into `src/url_handler.rs`
- Added `serde_json` dependency for JSON API parsing
- `SubmitUrl` now delegates to `url_handler::resolve_url()` which auto-detects content type and parses accordingly
- Multiple-channel sources (SomaFM, Radio Browser) create full station groups
- Single-channel or uncategorised sources add to "Uncategorised" group with `auto_add = true`
- Added `ConfigManager::add_group()` for bulk group insertion

### Files Changed
- `data/add_station.svg` — new SVG icon for the add station button
- `data/fm_radio_icon.svg` — new radio icon asset
- `data/station_artwork_placeholder.png` — new placeholder artwork for stations without artwork
- `data/com.system76.CosmicRadio.desktop` — removed (moved to project root)
- `data/radio_icon.svg` — removed (replaced by fm_radio_icon.svg)
- `src/app.rs` — all marquee, layout, Play from URL, artwork placeholder, and play/stop button changes
- `src/config.rs` — `add_station()` → `add_to_group(station, group_name)`, added `add_group()`
- `src/url_handler.rs` — new file (URL resolution, SomaFM/Radio Browser/PLS parsing)
- `Cargo.toml` — added `serde_json` dependency

### Current Architecture
```
src/
├── main.rs      (8 lines)   — cosmic::applet::run::<RadioApp>
├── config.rs    (155 lines) — ConfigManager, types, TOML I/O
├── audio.rs     (100 lines) — AudioBackend, GStreamer, PLS, metadata
├── artwork.rs   (89 lines)  — ArtworkCache, async download, hashing
├── app.rs       (537 lines) — RadioApp, Message, UI, cosmic::Application
└── url_handler.rs (326 lines) — URL resolution, SomaFM/Radio Browser/PLS/raw audio
```

## Session 5 — 19 May 2026

### Changes Made This Session

#### Pop OS readiness — architecture overhaul for official inclusion

Prepared the applet for inclusion in Pop OS as an official tray applet. All changes follow the conventions established by `pop-os/cosmic-applets`.

#### SVG icon compliance (symbolic naming + theming)

All SVG files were renamed with the `-symbolic` suffix per the freedesktop/COSMIC symbolic icon convention, and moved into the proper hicolor directory structure:

```
data/icons/scalable/
├── apps/
│   └── com.system76.CosmicRadio-symbolic.svg   (panel applet icon)
└── status/
    ├── play-button-symbolic.svg                 (play control)
    ├── stop-button-symbolic.svg                 (stop control)
    ├── add-station-symbolic.svg                 (add station action)
    └── mic-symbolic.svg                         (artwork placeholder)
```

Every SVG's `stroke` and `fill` attributes were changed from hardcoded `"white"`/`"#888888"` to `"currentColor"`, making icons properly respond to the OS theme (light/dark/high-contrast).

#### Desktop file

Created `data/com.system76.CosmicRadio.desktop` following the official COSMIC applet desktop entry pattern with:
- `Icon=com.system76.CosmicRadio-symbolic`
- `X-CosmicApplet=true`, `X-CosmicShrinkable=true`, `X-CosmicHoverPopup=Auto`, `X-OverflowPriority=10`
- `NoDisplay=true` (hides from app launcher — it's a panel applet)

#### AppStream metadata

Created `data/com.system76.CosmicRadio.metainfo.xml` required for Pop!_OS Software Center listing.

#### Justfile overhaul

Replaced the ad-hoc install script with a proper `appid`-based justfile matching COSMIC applet standards:
- `build`, `install`, `run`, `clean`, `check` (clippy), `all` recipes
- `install` uses `install -Dm0644` for proper destdir/rootdir support
- Installs to correct hicolor paths: `hicolor/scalable/apps/` and `hicolor/scalable/status/`

#### Debian packaging

Created full `debian/` directory for `dpkg-buildpackage`:
- `debian/control` — build deps (cargo, rustc, libgstreamer, libwayland, etc.) and runtime deps
- `debian/rules` — delegates to `just build` / `just install`
- `debian/changelog` — initial release entry
- `debian/copyright` — GPL-3.0-or-later

#### Code updates

- `src/app.rs` — updated `include_bytes!` paths to new SVG locations; changed `icon_button("radio_icon")` → `icon_button("com.system76.CosmicRadio-symbolic")`
- `src/config.rs` — removed unused `group_exists()` method to eliminate the only build warning
- `.gitignore` — added editor backup patterns (`*~`, `*.swp`, `*.swo`) and debian build artifacts

#### Cleanup

- Removed 5 backup files (`*.svg~`, `cosmic-radio.desktop~`) and unused `station_artwork_placeholder.png`
- Removed old SVGs from `data/` root (now living under `data/icons/scalable/`)

### Files Changed
- `data/icons/scalable/apps/com.system76.CosmicRadio-symbolic.svg` — new (was `data/fm_radio_icon.svg`)
- `data/icons/scalable/status/play-button-symbolic.svg` — new (was `data/play_button.svg`)
- `data/icons/scalable/status/stop-button-symbolic.svg` — new (was `data/stop_button.svg`)
- `data/icons/scalable/status/add-station-symbolic.svg` — new (was `data/add_station.svg`)
- `data/icons/scalable/status/mic-symbolic.svg` — new (was `data/cosmic-broadcast-mic-symbolic.svg`)
- `data/com.system76.CosmicRadio.desktop` — new file
- `data/com.system76.CosmicRadio.metainfo.xml` — new file
- `debian/control` — new file
- `debian/rules` — new file
- `debian/changelog` — new file
- `debian/copyright` — new file
- `debian/source/format` — new file
- `justfile` — rewritten with official COSMIC patterns
- `src/app.rs` — SVG path references, icon button name updated
- `src/config.rs` — removed unused `group_exists()` method
- `.gitignore` — added backup file and debian artifact patterns
- `data/*.svg~` — deleted (backup files)
- `data/*.desktop~` — deleted (backup files)
- `data/station_artwork_placeholder.png` — deleted (unused artifact)

### Current Architecture
```
data/
├── icons/scalable/
│   ├── apps/
│   │   └── com.system76.CosmicRadio-symbolic.svg
│   └── status/
│       ├── play-button-symbolic.svg
│       ├── stop-button-symbolic.svg
│       ├── add-station-symbolic.svg
│       └── mic-symbolic.svg
├── com.system76.CosmicRadio.desktop
├── com.system76.CosmicRadio.metainfo.xml
└── stations.toml
debian/
├── changelog
├── control
├── copyright
├── rules
└── source/format
src/
├── main.rs         (9 lines)   — entry point
├── app.rs          (537 lines) — RadioApp, Message, UI
├── config.rs       (153 lines) — ConfigManager, types, TOML I/O
├── audio.rs        (100 lines) — AudioBackend, GStreamer, PLS
├── artwork.rs      (89 lines)  — ArtworkCache, async download
└── url_handler.rs  (326 lines) — URL resolution, API parsers
```
