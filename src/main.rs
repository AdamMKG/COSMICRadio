use cosmic::{
    Element, Task, app,
    iced::{
        self, Alignment,
        platform_specific::shell::commands::popup::{destroy_popup, get_popup},
        widget::{column},
        window,
    },
    widget::{text},
};

const APP_ID: &str = "com.system76.CosmicRadio";

fn main() -> cosmic::iced::Result {
    cosmic::applet::run::<RadioApp>(())
}

struct RadioApp {
    core: cosmic::app::Core,
    popup: Option<window::Id>,
}

#[derive(Debug, Clone)]
enum Message {
    TogglePopup,
    Closed(window::Id),
}

impl cosmic::Application for RadioApp {
    type Executor = cosmic::SingleThreadExecutor;
    type Flags = ();
    type Message = Message;
    const APP_ID: &'static str = APP_ID;

    fn core(&self) -> &cosmic::app::Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut cosmic::app::Core {
        &mut self.core
    }

    fn init(core: cosmic::app::Core, _flags: ()) -> (Self, app::Task<Self::Message>) {
        (
            Self {
                core,
                popup: None,
            },
            Task::none(),
        )
    }

    fn on_close_requested(&self, id: window::Id) -> Option<Message> {
        Some(Message::Closed(id))
    }

    fn update(&mut self, message: Self::Message) -> app::Task<Self::Message> {
        match message {
            Message::TogglePopup => {
                if let Some(p) = self.popup.take() {
                    return destroy_popup(p);
                } else {
                    let new_id = window::Id::unique();
                    self.popup.replace(new_id);

                    let popup_settings = self.core.applet.get_popup_settings(
                        self.core.main_window_id().unwrap(),
                        new_id,
                        None,
                        None,
                        None,
                    );

                    return get_popup(popup_settings);
                }
            }
            Message::Closed(id) => {
                if self.popup == Some(id) {
                    self.popup = None;
                }
            }
        }
        Task::none()
    }

    fn view(&self) -> Element<'_, Message> {
        self.core
            .applet
            .icon_button("radio-symbolic")
            .on_press_down(Message::TogglePopup)
            .into()
    }

    fn view_window(&self, id: window::Id) -> Element<'_, Message> {
        if matches!(self.popup, Some(p) if p == id) {
            let content = column![
                text::title3("Hello World"),
            ]
            .align_x(Alignment::Center)
            .padding(16);

            self.core.applet.popup_container(content).into()
        } else {
            column![].into()
        }
    }

    fn style(&self) -> Option<iced::theme::Style> {
        Some(cosmic::applet::style())
    }
}
