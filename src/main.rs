mod app;
mod artwork;
mod audio;
mod config;
mod mpris;
mod url_handler;

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<app::RadioApp>(())
}
