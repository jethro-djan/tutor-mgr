pub mod domain;
pub mod dashboard;
pub mod shell;
pub mod icons;
pub mod ui_components;
pub mod students;

mod app;

use iced::Size;

use crate::app::App;

fn main() -> iced::Result {
    iced::application(
        App::new,
        App::update,
        App::view,
    )
    .title(App::title)
    .subscription(App::subscription)
    .window(iced::window::Settings {
        size: Size::new(1200.0, 800.0),
        maximized: false,
        fullscreen: false,
        min_size: Some(Size::new(900.0, 700.0)),
        resizable: true,
        closeable: true,
        minimizable: true,
        icon: None,
        exit_on_close_request: true,
        ..Default::default()
    })
    .run()
}
