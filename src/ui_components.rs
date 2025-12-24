use iced::widget::{row, text};
use iced::{Element, Font};
use iced::advanced::graphics::core::font;

pub fn page_header<'a, Message: 'a>(header_text: &'a str) -> Element<'a, Message> {
    let page_title_text = text(header_text)
        .font(Font {
            weight: font::Weight::Bold,
            ..Default::default()
        })
        .size(24);
    
    row![page_title_text].into()
}
