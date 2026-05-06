Project: COSMIC Radio Applet
1. Core Vision

A minimalist, native COSMIC panel applet for streaming internet radio. The application has no main window. Interaction occurs exclusively via a click-triggered popup menu from the COSMIC panel.
2. Technical Stack

    Language: Rust

    Framework: libcosmic (with cosmic-applet features)

    UI Engine: Iced (integrated into libcosmic)

    Audio Backend: rodio or gst-play (lightweight streaming)

    Configuration: Plaintext/TOML file for station management.

3. Architecture Requirements

    Applet Lifecycle: Use the cosmic_applet implementation. Do not use cosmic::Application which creates a standard window.

    View Logic:

        Status Area: Display a simple radio icon in the panel.

        Popup Menu: A cosmic::widget::column containing:

            Currently playing station (bold/highlighted).

            Play/Stop toggle button.

            Volume slider.

            A scrollable list of stations.

            A "Edit Stations" button at the bottom.

4. Specific Feature Constraints (Minimalist)

    Station Storage:

        Path: ~/.config/cosmic-radio/stations.csv or stations.toml.

        Format: Station Name, Stream URL.

    No GUI Editor: The "Edit Stations" button must not open a custom UI. It should simply trigger xdg-open on the configuration file using the system's default text editor.

    Persistence: The applet should remember the last played station and volume level upon restart.

5. Development Roadmap (AI Instructions)

    Phase 1: Scaffold a basic cosmic_applet that renders a "Hello World" popup.

    Phase 2: Implement a watcher for the stations.toml file so the list updates in real-time when the user saves changes in their text editor.

    Phase 3: Integrate the audio playback engine to handle stream buffering and state (Playing/Paused).

    Phase 4: Style the popup using libcosmic widgets to ensure native look-and-feel (theming, corner radius, and padding).

Implementation Tip for the Agent

    Warning: Strictly avoid std::window. If the code attempts to initialize a windowing context outside of the Layer Shell for the panel, it is incorrect. The UI must be returned via the view() function of the Applet trait.
