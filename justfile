# Build the applet in release mode
build:
    cargo build --release

# Install the applet to user local directories
install: build
    mkdir -p ~/.local/bin
    cp target/release/cosmic-radio ~/.local/bin/
    mkdir -p ~/.local/share/applications
    cp data/com.system76.CosmicRadio.desktop ~/.local/share/applications/
    mkdir -p ~/.local/share/icons/hicolor/scalable/apps
    cp data/radio_icon.svg ~/.local/share/icons/hicolor/scalable/apps/radio_icon.svg

# Run the applet (for testing)
run:
    cargo run

# Clean build artifacts
clean:
    cargo clean

# Build and install in one step
all: install
    @echo "COSMIC Radio applet installed. Log out/in or restart COSMIC panel to see it."
