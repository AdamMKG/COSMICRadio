# Build the applet in release mode
build:
    cargo build --release

# Install the applet to system directories (requires sudo for /usr/share)
install: build
    mkdir -p ~/.local/bin
    cp target/release/cosmic-radio ~/.local/bin/
    mkdir -p ~/.local/share/applications
    cp data/com.system76.CosmicRadio.desktop ~/.local/share/applications/
    mkdir -p ~/.local/share/icons/hicolor/scalable/apps
    cp data/radio_icon.svg ~/.local/share/icons/hicolor/scalable/apps/radio_icon.svg
    # Install default stations.toml to system share directory
    sudo mkdir -p /usr/share/cosmic-radio
    sudo cp data/stations.toml /usr/share/cosmic-radio/stations.toml

# Run the applet (for testing)
run:
    cargo run

# Clean build artifacts
clean:
    cargo clean

# Build and install in one step
all: install
    @echo "COSMIC Radio applet installed. Log out/in or restart COSMIC panel to see it."
