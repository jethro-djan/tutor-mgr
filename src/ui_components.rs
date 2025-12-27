use iced::advanced::graphics::core::font;
use iced::widget::{Button, Container, button, container, svg};
use iced::widget::{Row, row, text};
use iced::{Background, Border, Center, Color, Element, Font, Theme};

pub fn page_header<'a, Message: 'a>(header_text: &'a str) -> Row<'a, Message> {
    let page_title_text = text(header_text)
        .font(Font {
            weight: font::Weight::Bold,
            ..Default::default()
        })
        .size(24);

    row![page_title_text].padding([35, 30])
}

pub fn ui_button<'a, Message: 'a>(
    btn_text: &'a str,
    btn_text_size: f32,

    icon_svg_handle: svg::Handle,
    icon_width: f32,
    icon_height: f32,

    cn_color_fn: impl Fn(&Theme) -> Color + 'a + Copy,
    bg_color_fn: impl Fn(&Theme) -> Color + 'a,
) -> Button<'a, Message> {
    button(
        container(
            row![
                svg::Svg::new(icon_svg_handle)
                    .width(icon_width)
                    .height(icon_height)
                    .style(move |theme: &Theme, _status: svg::Status| {
                        svg::Style {
                            color: Some(cn_color_fn(theme)),
                        }
                    }),
                text(btn_text)
                    .size(btn_text_size)
                    .font(Font {
                        weight: font::Weight::Semibold,
                        ..Default::default()
                    })
                    .style(move |theme: &Theme| {
                        text::Style {
                            color: Some(cn_color_fn(theme)),
                        }
                    }),
            ]
            .spacing(5)
            .align_y(Center),
        )
        .align_x(Center),
    )
    .style(
        move |theme: &Theme, _status: button::Status| button::Style {
            background: Some(Background::Color(bg_color_fn(theme))),
            border: Border {
                radius: 10.0.into(),
                ..Default::default()
            },
            ..Default::default()
        },
    )
}

pub fn global_content_container<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> Container<'a, Message> {
    container(content).padding([0, 30])
}
