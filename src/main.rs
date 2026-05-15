mod app;
mod artwork;
mod audio;
mod config;

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<app::RadioApp>(())
}
