appid := "com.system76.CosmicRadio"
name := "cosmic-radio"
prefix := "/usr"
bindir := prefix / "bin"
datadir := prefix / "share"
iconsdir := datadir / "icons" / "hicolor" / "scalable"
desktopdir := datadir / "applications"
metainfodir := datadir / "metainfo"
system_stationsdir := datadir / name / "stations.toml"
rootdir := ""

# Build in release mode
build:
    cargo build --release

# Install to system directories
install: build
    install -Dm0755 target/release/{{ name }} {{ rootdir }}{{ bindir }}/{{ name }}
    install -Dm0644 data/{{ appid }}.desktop {{ rootdir }}{{ desktopdir }}/{{ appid }}.desktop
    install -Dm0644 data/{{ appid }}.metainfo.xml {{ rootdir }}{{ metainfodir }}/{{ appid }}.metainfo.xml
    install -Dm0644 data/icons/scalable/apps/{{ appid }}-symbolic.svg {{ rootdir }}{{ iconsdir }}/apps/{{ appid }}-symbolic.svg
    install -Dm0644 data/icons/scalable/status/play-button-symbolic.svg {{ rootdir }}{{ iconsdir }}/status/play-button-symbolic.svg
    install -Dm0644 data/icons/scalable/status/stop-button-symbolic.svg {{ rootdir }}{{ iconsdir }}/status/stop-button-symbolic.svg
    install -Dm0644 data/icons/scalable/status/add-station-symbolic.svg {{ rootdir }}{{ iconsdir }}/status/add-station-symbolic.svg
    install -Dm0644 data/icons/scalable/status/mic-symbolic.svg {{ rootdir }}{{ iconsdir }}/status/mic-symbolic.svg
    install -Dm0644 data/stations.toml {{ rootdir }}{{ datadir }}/{{ name }}/stations.toml

# Run the applet (for testing)
run:
    cargo run

# Clean build artifacts
clean:
    cargo clean

# Run clippy lints
check:
    cargo clippy --all-targets -- -D warnings

# Build and install in one step
all: install
    @echo "{{ name }} installed. Log out/in or restart COSMIC panel to see it."

# Bump version in Cargo.toml, update debian/changelog, commit, and tag
tag version:
    sed -i '0,/^version/s/^version.*/version = "{{ version }}"/' Cargo.toml
    cargo check
    cargo clean
    dch -D noble -v {{ version }}-1
    git add Cargo.toml Cargo.lock debian/changelog
    git commit -m "release: v{{ version }}"
    git tag -a v{{ version }} -m "release v{{ version }}"
