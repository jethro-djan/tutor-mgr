use iced::window::frames;
use std::time::Instant;

use lilt::{Animated, Easing};

use iced::advanced::graphics::core::font;
use iced::mouse::Interaction;
use iced::widget::{Container, column, container, mouse_area, row, svg, text};
use iced::{Background, Border, Center, Color, Element, Font, Length, Subscription, Task, Theme};

use crate::icons;

pub struct ShellState {
    pub current_screen: Screen,
    pub selected_menu_item: SideMenuItem,
    pub hovered_menu_item: Option<SideMenuItem>,
    pub side_menu_hovered: bool,

    pub animated_menu_width_change: Animated<bool, Instant>,
    pub animated_menu_item_height_change: Animated<bool, Instant>,
    pub show_menu_text: bool,
}

impl Default for ShellState {
    fn default() -> Self {
        Self {
            current_screen: Screen::Dashboard,
            selected_menu_item: SideMenuItem::Dashboard,
            hovered_menu_item: None,
            side_menu_hovered: false,

            animated_menu_width_change: Animated::new(false)
                .duration(300.)
                .easing(Easing::EaseInOut),
            animated_menu_item_height_change: Animated::new(false)
                .duration(200.)
                .easing(Easing::EaseInOut),
            show_menu_text: false,
        }
    }
}

#[derive(Debug)]
pub enum Screen {
    Dashboard,
    StudentManager,
    Settings,
    Logout,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum SideMenuItem {
    Dashboard,
    StudentManager,
    Settings,
    Logout,
}

impl Into<Screen> for SideMenuItem {
    fn into(self) -> Screen {
        match self {
            SideMenuItem::Dashboard => Screen::Dashboard,
            SideMenuItem::StudentManager => Screen::StudentManager,
            SideMenuItem::Settings => Screen::Settings,
            SideMenuItem::Logout => Screen::Logout,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Msg {
    NavigateTo(SideMenuItem),
    MenuItemHovered(Option<SideMenuItem>),
    SideMenuHovered(bool),
    Tick,
}

pub fn update(state: &mut ShellState, msg: Msg) {
    match msg {
        Msg::NavigateTo(item) => {
            state.selected_menu_item = item;
            state.current_screen = item.into();
        }
        Msg::SideMenuHovered(is_hovered) => {
            let now = Instant::now();
            state.animated_menu_width_change.transition(is_hovered, now);

            state.side_menu_hovered = is_hovered;
        }
        Msg::MenuItemHovered(is_hovered_opt) => {
            state.hovered_menu_item = is_hovered_opt;
        }
        Msg::Tick => (),
    }
}

pub fn view<'a, Message: 'a>(
    state: &'a ShellState,
    content: Element<'a, Message>,
    map_msg: impl Fn(Msg) -> Message + 'a,
) -> Element<'a, Message> {
    row![view_side_menu(state).map(map_msg), container(content)]
        // .spacing(20)
        .into()
}

fn view_side_menu<'a>(state: &'a ShellState) -> Element<'a, Msg> {
    let now = Instant::now();

    mouse_area(
        container(
            column![
                view_logo(state),
                column![
                    menu_item(
                        "Dashboard",
                        icons::dashboard(),
                        SideMenuItem::Dashboard,
                        state,
                        now
                    ),
                    menu_item(
                        "Student Manager",
                        icons::student_manager(),
                        SideMenuItem::StudentManager,
                        state,
                        now
                    ),
                ]
                .spacing(5),
                container(
                    column![
                        menu_item(
                            "Settings",
                            icons::settings(),
                            SideMenuItem::Settings,
                            state,
                            now
                        ),
                        menu_item("Logout", icons::logout(), SideMenuItem::Logout, state, now),
                    ]
                    .spacing(5)
                )
                .align_bottom(Length::Fill)
            ]
            .spacing(20),
        )
        .padding([20, 0])
        .width(
            state
                .animated_menu_width_change
                .animate_bool(70.0, 180.0, now),
        )
        .height(Length::Fill)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();

            container::Style {
                background: Some(palette.background.weak.color.into()),
                border: Border {
                    color: palette.background.strong.color,
                    width: 1.0,
                    radius: 0.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        }),
    )
    .on_enter(Msg::SideMenuHovered(true))
    .on_exit(Msg::SideMenuHovered(false))
    .into()
}

fn view_logo(state: &ShellState) -> Element<'_, Msg> {
    let logo_handle = if state.side_menu_hovered {
        icons::logo_expanded()
    } else {
        icons::logo()
    };

    let logo = svg(logo_handle)
        .width(if state.side_menu_hovered { 140 } else { 40 })
        .height(40);

    container(logo)
        .center_x(Length::Fill)
        .padding([10, 0])
        .into()
}

fn menu_item<'a>(
    menu_name: &'a str,
    icon_handle: svg::Handle,
    item_selected: SideMenuItem,
    state: &'a ShellState,
    now: Instant,
) -> Element<'a, Msg> {
    let is_selected = |item_selected| state.selected_menu_item == item_selected;
    let is_hovered = |item_selected| state.hovered_menu_item == Some(item_selected);

    let icon = svg::Svg::new(icon_handle).width(25).height(25).style(
        move |_theme: &Theme, _status: svg::Status| menu_icon_style(is_hovered(item_selected)),
    );

    mouse_area(menu_item_container(
        icon,
        menu_name,
        is_selected(item_selected),
        is_hovered(item_selected),
        state.side_menu_hovered,
        &state.animated_menu_item_height_change,
        now,
    ))
    .interaction(Interaction::Pointer)
    .on_press(Msg::NavigateTo(item_selected))
    .on_enter(Msg::MenuItemHovered(Some(item_selected)))
    .on_exit(Msg::MenuItemHovered(None))
    .into()
}

fn menu_icon_style(is_item_hovered: bool) -> svg::Style {
    if is_item_hovered {
        svg::Style {
            color: Some(Color {
                r: 0.1,
                g: 0.1,
                b: 1.0,
                a: 0.9,
            }),
        }
    } else {
        svg::Style { color: None }
    }
}

fn menu_item_container<'a>(
    item: svg::Svg<'a>,
    item_text: &'a str,
    is_item_selected: bool,
    is_item_hovered: bool,
    is_side_menu_hovered: bool,
    animated_container_height: &Animated<bool, Instant>,
    now: Instant,
) -> Container<'a, Msg> {
    let create_text = move |is_hovered: bool, is_selected: bool| {
        text(item_text)
            .font(Font {
                weight: font::Weight::Light,
                ..Default::default()
            })
            .size(11)
            .wrapping(text::Wrapping::None)
            .style(move |theme: &Theme| {
                if is_hovered {
                    text::Style {
                        color: Some(Color {
                            r: 0.1,
                            g: 0.1,
                            b: 1.0,
                            a: 0.9,
                        }),
                    }
                } else if is_selected {
                    text::Style {
                        color: Some(theme.extended_palette().background.strong.text),
                    }
                } else {
                    text::Style::default()
                }
            })
    };

    let content = if is_item_hovered || is_side_menu_hovered {
        row![item, create_text(is_item_hovered, is_item_selected)]
            .align_y(Center)
            .spacing(10)
    } else {
        row![item,].spacing(10)
    };

    container(content)
        .width(Length::Fill)
        .align_left(Length::Fill)
        .center_y(Length::Fixed(
            animated_container_height.animate_bool(40.0, 45.0, now),
        ))
        .padding([0, 20])
        .style(move |theme: &Theme| {
            if is_item_selected {
                container::Style {
                    text_color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.5)),
                    background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.5))),
                    ..Default::default()
                }
            } else {
                container::transparent(theme)
            }
        })
}

pub fn subscription(state: &ShellState) -> Subscription<Msg> {
    let now = Instant::now();
    if state.animated_menu_width_change.in_progress(now) {
        frames().map(|_| Msg::Tick)
    } else {
        Subscription::none()
    }
}
