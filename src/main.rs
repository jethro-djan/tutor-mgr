use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, TimeZone, Weekday};
use iced::advanced::graphics::core::font;
use iced::mouse::Interaction;
use iced::widget::{
    Column, Container, Row, button, column, container, focus_next, mouse_area, row, stack, svg,
    text, text_input, canvas,
};
use iced::{
    Alignment, Background, Border, Center, Color, Element, Font, Length, Shadow, Task, Theme,
    Vector, Renderer, Rectangle,
};
use lilt::{Animated, Easing};
use std::time::Instant;

// =========================================
// DOMAIN MODELS AND LOGIC 
// =========================================
struct Student {
    name: PersonalName,
    subject: TutorSubject,
    tabled_sessions: Vec<SessionData>,
    actual_sessions: Vec<DateTime<Local>>,

    payment_data: PaymentData,
}

struct PersonalName {
    first: String,
    last: String,
    other: Option<String>,
}

struct SessionData {
    day: Weekday,
    time: String,
}

enum TutorSubject {
    AdditionalMathematics,
    ExtendedMathematics,
    Statistics,
}

impl TutorSubject {
    fn as_str(&self) -> &str {
        match self {
            TutorSubject::AdditionalMathematics => "Additional Mathematics",
            TutorSubject::ExtendedMathematics => "Extended Mathematics",
            TutorSubject::Statistics => "Statistics",
        }
    }
}

struct PaymentData {
    per_session_amount: f32,
}

fn compute_accrued_amount(student: &Student) -> f32 {
    let no_of_days = compute_num_of_completed_sessions(student);
    student.payment_data.per_session_amount * (no_of_days as f32)
}

fn compute_num_of_completed_sessions(student: &Student) -> i32 {
    let today_date = Local::now().naive_local().date();
    let current_year = today_date.year();
    let current_month = today_date.month();
    let month_start_date = NaiveDate::from_ymd_opt(current_year, current_month, 1).unwrap();

    let duration = today_date.signed_duration_since(month_start_date);
    let all_dates: Vec<NaiveDate> = (0..=duration.num_days())
        .map(|i| month_start_date + Duration::days(i))
        .collect();
    let session_days: Vec<Weekday> = student
        .tabled_sessions
        .iter()
        .map(|session| session.day)
        .collect();

    let actual_session_dates: Vec<NaiveDate> = student
        .actual_sessions
        .iter()
        .map(|dt| dt.naive_local().date())
        .collect();

    let session_dates: Vec<&NaiveDate> = all_dates
        .iter()
        .filter(|date| actual_session_dates.contains(date))
        .collect();

    let no_of_days = session_dates
        .iter()
        .filter(|date| session_days.contains(&date.weekday()))
        .count();

    no_of_days as i32
}

fn get_next_session(student: &Student) -> NaiveDate {
    let tabled_next_days: Vec<Weekday> = student
        .tabled_sessions
        .iter()
        .map(|session| session.day)
        .collect();

    let today = Local::now().naive_local().date();
    let next_seven_dates: Vec<NaiveDate> = (1..=7).map(|i| today + Duration::days(i)).collect();

    next_seven_dates
        .into_iter()
        .filter(|date| tabled_next_days.contains(&date.weekday()))
        .min()
        .unwrap()
}

// =========================================
// APPLICATION STATE
// =========================================
struct TutoringManager {
    current_screen: Screen,
    state: State,
}

struct State {
    // Navigation
    selected_menu_item: SideMenuItem,
    hovered_menu_item: Option<SideMenuItem>,
    side_menu_hovered: bool,

    // Side Menu Layout
    side_menu_width: f32,
    menu_item_container_height: f32,
    side_menu_target_width: f32,
    animated_menu_width_change: Animated<bool, Instant>,
    show_menu_text: bool,

    // Dashboard State
    dashboard_card_hovered: bool,
    hovered_dashboard_card: Option<usize>,

    // StudentManager State
    search_query: String,
    show_add_student_modal: bool,
    students: Vec<Student>,
    hovered_student_card: Option<usize>,
}

#[derive(Debug)]
enum Screen {
    Dashboard,
    StudentManager,
}

#[derive(Debug, Clone)]
enum SideMenuItem {
    Dashboard,
    StudentManager,
}

#[derive(Debug, Clone)]
enum Message {
    // Navigation
    NavigateToScreen(SideMenuItem),
    MenuItemHovered(Option<SideMenuItem>),
    SideMenuHovered(bool),

    // Dashboard
    // DashboardCardHovered(bool),
    DashboardCardHovered(Option<usize>),

    // Student Manager
    ShowAddStudentModal,
    CloseAddStudentModal,
    StudentCardHovered(Option<usize>),
}

// =========================================
// APPLICATION LOGIC
// =========================================

impl TutoringManager {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                current_screen: Screen::Dashboard,
                state: State {
                    selected_menu_item: SideMenuItem::Dashboard,
                    hovered_menu_item: None,
                    side_menu_hovered: false,

                    side_menu_width: 50.0,
                    side_menu_target_width: 50.0,
                    animated_menu_width_change: Animated::new(false).duration(300.).easing(Easing::EaseOut),
                    menu_item_container_height: 50.0,
                    show_menu_text: false,

                    dashboard_card_hovered: false,
                    hovered_dashboard_card: None,

                    search_query: String::new(),
                    show_add_student_modal: false,
                    students: mock_data(),
                    hovered_student_card: None,
                },
            },
            Task::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Tutor Manager")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Navigation Messages
            Message::NavigateToScreen(menu_item) => self.handle_navigation(menu_item),
            Message::MenuItemHovered(menu_item_opt) => self.handle_menu_item_hover(menu_item_opt),
            Message::SideMenuHovered(is_hovered) => self.handle_side_menu_hover(is_hovered),

            // Dashboard Messages
            // Message::DashboardCardHovered(is_hovered) => {
            //     self.handle_dashboard_card_hover(is_hovered)
            // }
            Message::DashboardCardHovered(card_index) => {
                self.state.hovered_dashboard_card = card_index;
                Task::none()
            }

            // StudentManager Messages
            Message::ShowAddStudentModal => self.handle_show_add_student_modal(),
            Message::CloseAddStudentModal => self.handle_close_add_student_modal(),
            Message::StudentCardHovered(card_index) => self.handle_student_card_hover(card_index),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let main_content = container(self.view_current_screen()).padding([50,20]);

        let layout = row![self.view_side_menu(), main_content,].spacing(20);

        container(layout).into()
    }

    fn view_current_screen(&self) -> Element<'_, Message> {
        match self.current_screen {
            Screen::Dashboard => self.view_dashboard(),
            Screen::StudentManager => self.view_student_manager(),
        }
    }
}

// Update helpers
impl TutoringManager {
    fn handle_navigation(&mut self, menu_item: SideMenuItem) -> Task<Message> {
        self.state.selected_menu_item = menu_item.clone();
        self.current_screen = match menu_item {
            SideMenuItem::Dashboard => Screen::Dashboard,
            SideMenuItem::StudentManager => Screen::StudentManager,
        };
        Task::none()
    }

    fn handle_menu_item_hover(&mut self, menu_item_opt: Option<SideMenuItem>) -> Task<Message> {
        self.state.hovered_menu_item = menu_item_opt.clone();
        Task::none()
    }

    fn handle_side_menu_hover(&mut self, is_hovered: bool) -> Task<Message> {
        if is_hovered {
            self.state.menu_item_container_height = 90.0;
            self.state.side_menu_width = 90.0;
            self.state.side_menu_hovered = true;
            self.state.show_menu_text = true;
        } else {
            self.state.menu_item_container_height = 50.0;
            self.state.side_menu_width = 50.0;
            self.state.side_menu_hovered = false;
            self.state.show_menu_text = false;
        }
        Task::none()
    }

    fn handle_dashboard_card_hover(&mut self, is_hovered: bool) -> Task<Message> {
        self.state.dashboard_card_hovered = is_hovered;
        Task::none()
    }

    fn handle_show_add_student_modal(&mut self) -> Task<Message> {
        self.state.show_add_student_modal = true;
        focus_next()
    }

    fn handle_close_add_student_modal(&mut self) -> Task<Message> {
        self.state.show_add_student_modal = false;
        Task::none()
    }

    fn handle_student_card_hover(&mut self, card_index: Option<usize>) -> Task<Message> {
        self.state.hovered_student_card = card_index;
        Task::none()
    }
}

// Top-level Views
impl TutoringManager {
    fn view_dashboard(&self) -> Element<'_, Message> {
        struct CardInfo<'a> {
            title: &'a str,
            value: &'a str,
            trend: Option<(&'a str, bool)>,
            hovered_dashboard: Option<usize>,
            variant: DashboardCardVariant,
        }

        let card_data = [
            CardInfo {
                title: "Attendance Rate",
                value: "85",
                trend: Some(("5%, MoM", true)),
                hovered_dashboard: self.state.hovered_dashboard_card,
                variant: DashboardCardVariant::Attendance,
            },
            CardInfo {
                title: "Actual Earnings",
                value: "GHS 1500",
                trend: None,
                hovered_dashboard: self.state.hovered_dashboard_card,
                variant: DashboardCardVariant::ActualEarnings,
            },
            CardInfo {
                title: "Potential Earnings",
                value: "GHS 2000",
                trend: None,
                hovered_dashboard: self.state.hovered_dashboard_card,
                variant: DashboardCardVariant::PotentialEarnings,
            },
            CardInfo {
                title: "Revenue Lost",
                value: "GHS 500",
                trend: Some(("3%, MoM", false)),
                hovered_dashboard: self.state.hovered_dashboard_card,
                variant: DashboardCardVariant::RevenueLost,
            },
        ];

        let summary_cards = row(card_data.iter().enumerate().map(|(index, card)| {
            let is_hovered = card.hovered_dashboard == Some(index);
            metric_card(card.title, card.value, card.trend, is_hovered, Some(index), card.variant)
        })).spacing(10);

        let attendance_trend_chart = container(text("Trend chart"));
        let potential_vs_actual_chart = container(text("Bar chart"));

        let graphs = row![attendance_trend_chart, potential_vs_actual_chart,];

        container(
            Column::new()
                .spacing(30)
                .push(self.view_page_header("Dashboard"))
                .push(summary_cards)
                .push(graphs),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn view_student_manager(&self) -> Element<'_, Message> {
        let search_bar = self.view_search_bar("Search Students", &self.state.search_query);
        let add_button =
            button(svg(icons::plus()).width(25).height(25)).on_press(Message::ShowAddStudentModal);
        let action_bar = row![search_bar, add_button].spacing(100);
        let card_container = container(
            Row::new()
                .extend(self.view_student_manager_card_list())
                .spacing(30),
        );

        let main_container = container(
            column![
                self.view_page_header("Student Manager"),
                action_bar,
                card_container,
            ]
            .spacing(30),
        )
        .width(Length::Fill)
        .height(Length::Fill);

        if self.state.show_add_student_modal {
            modal(main_container.into(), || {
                container(column![
                    row![text!("Modal open")],
                    button(text!("Close modal")).on_press(Message::CloseAddStudentModal),
                ])
                .into()
            })
            .into()
        } else {
            main_container.into()
        }
    }
}

// Component views
impl TutoringManager {
    fn view_page_header(&self, header_text: &str) -> Element<'_, Message> {
        let page_title_text = text(format!("{}", header_text))
            .size(20)
            .font(Font {
                weight: font::Weight::Bold,
                ..Default::default()
            })
            .size(24);
        let page_title = row![page_title_text];
        row![page_title].into()
    }

    fn view_side_menu(&self) -> Element<'_, Message> {
        let is_dash_selected = match self.state.selected_menu_item {
            SideMenuItem::Dashboard => true,
            SideMenuItem::StudentManager => false,
        };
        let is_student_selected = match self.state.selected_menu_item {
            SideMenuItem::Dashboard => false,
            SideMenuItem::StudentManager => true,
        };
        let is_dash_hovered = match self.state.hovered_menu_item {
            Some(SideMenuItem::Dashboard) => true,
            Some(SideMenuItem::StudentManager) => false,
            None => false,
        };
        let is_student_hovered = match self.state.hovered_menu_item {
            Some(SideMenuItem::Dashboard) => false,
            Some(SideMenuItem::StudentManager) => true,
            None => false,
        };

        let dash_icon = svg::Svg::new(icons::dashboard().clone())
            .width(25)
            .height(25)
            .style(move |_theme: &Theme, _status: svg::Status| {
                styles::menu_icon_style(is_dash_hovered)
            });
        let dash_item_text = String::from("Dashboard");

        let student_icon = svg::Svg::new(icons::student_manager().clone())
            .width(25)
            .height(25)
            .style(move |_theme: &Theme, _status: svg::Status| {
                styles::menu_icon_style(is_student_hovered)
            });
        let student_item_text = String::from("Student Manager");

        mouse_area(
            container(
                column![
                    mouse_area(menu_item_container(
                        dash_icon,
                        dash_item_text,
                        is_dash_selected,
                        is_dash_hovered,
                        self.state.side_menu_hovered,
                        self.state.menu_item_container_height,
                    ))
                    .interaction(Interaction::Pointer)
                    .on_press(Message::NavigateToScreen(SideMenuItem::Dashboard))
                    .on_enter(Message::MenuItemHovered(Some(SideMenuItem::Dashboard)))
                    .on_exit(Message::MenuItemHovered(None)),
                    mouse_area(menu_item_container(
                        student_icon,
                        student_item_text,
                        is_student_selected,
                        is_student_hovered,
                        self.state.side_menu_hovered,
                        self.state.menu_item_container_height,
                    ))
                    .interaction(Interaction::Pointer)
                    .on_press(Message::NavigateToScreen(SideMenuItem::StudentManager))
                    .on_enter(Message::MenuItemHovered(Some(SideMenuItem::StudentManager)))
                    .on_exit(Message::MenuItemHovered(None)),
                ]
                .spacing(20),
            )
            .padding([250, 0])
            .width(self.state.side_menu_width)
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
        .interaction(Interaction::Pointer)
        .on_enter(Message::SideMenuHovered(true))
        .on_exit(Message::SideMenuHovered(false))
        .into()
    }

    fn view_search_bar(
        &self,
        placeholder_text: &str,
        search_query_state_tracker: &str,
    ) -> Element<'_, Message> {
        container(text_input(placeholder_text, &search_query_state_tracker)).into()
    }

    fn view_student_manager_card_list(&self) -> Vec<Element<'_, Message>> {
        let card_list = self
            .state
            .students
            .iter()
            .enumerate()
            .map(|(index, student)| {
                let next_session = get_next_session(student);
                let day = next_session.format("%A").to_string();
                let date = next_session.format("%d %B %Y").to_string();

                let is_hovered = self.state.hovered_student_card == Some(index);

                let student_other_name = match &student.name.other {
                    Some(name) => name,
                    None => "",
                };

                let title_section = row![
                    column![
                        container(if student.name.other == None {
                            text(format!("{} {}", student.name.first, student.name.last))
                                .font(Font {
                                    weight: font::Weight::Bold,
                                    ..Default::default()
                                })
                                .size(20)
                        } else {
                            text(format!(
                                "{} {} {}",
                                student.name.first, student_other_name, student.name.last
                            ))
                            .font(Font {
                                weight: font::Weight::Bold,
                                ..Default::default()
                            })
                            .size(20)
                        }),
                        container(
                            text(student.subject.as_str())
                                .font(Font {
                                    weight: font::Weight::Light,
                                    ..Default::default()
                                })
                                .size(15)
                        ),
                    ]
                    .align_x(Alignment::Start)
                    .width(Length::Fill)
                    .spacing(5)
                ]
                .height(Length::Fixed(50.0));

                let main_section = column![
                    row![
                        container(
                            svg::Svg::new(icons::calendar().clone())
                                .width(22)
                                .height(22)
                        )
                        .align_y(Alignment::Center)
                        .height(Length::Fixed(30.0)),
                        container(
                            column![
                                text("Schedule")
                                    .font(Font {
                                        weight: font::Weight::Normal,
                                        ..Default::default()
                                    })
                                    .size(12),
                                Column::new()
                                    .extend(student.tabled_sessions.iter().map(|session| {
                                        text(format!(
                                            "{} {}",
                                            session.day.to_string(),
                                            session.time
                                        ))
                                        .into()
                                    }))
                                    .spacing(2),
                            ]
                            .spacing(4)
                        ),
                    ]
                    .spacing(10),
                    row![
                        container(
                            svg::Svg::new(icons::schedule().clone())
                                .width(22)
                                .height(22)
                        )
                        .align_y(Alignment::Center)
                        .height(Length::Fixed(30.0)),
                        container(
                            column![
                                text("Next session")
                                    .font(Font {
                                        weight: font::Weight::Normal,
                                        ..Default::default()
                                    })
                                    .size(12),
                                text(format!("{}, {}", day, date))
                            ]
                            .spacing(5),
                        ),
                    ]
                    .spacing(10),
                    row![
                        container(
                            svg::Svg::new(icons::check_circle().clone())
                                .width(22)
                                .height(22)
                        )
                        .align_y(Alignment::Center)
                        .height(Length::Fixed(30.0)),
                        container(
                            column![
                                text("Completed sessions")
                                    .font(Font {
                                        weight: font::Weight::Normal,
                                        ..Default::default()
                                    })
                                    .size(12),
                                text(format!("{}", compute_num_of_completed_sessions(student))),
                            ]
                            .spacing(5),
                        ),
                    ]
                    .spacing(10),
                    row![
                        container(
                            svg::Svg::new(icons::payments().clone())
                                .width(22)
                                .height(22)
                        )
                        .align_y(Alignment::Center)
                        .height(Length::Fixed(30.0)),
                        container(
                            column![
                                text("Amount accrued")
                                    .font(Font {
                                        weight: font::Weight::Normal,
                                        ..Default::default()
                                    })
                                    .size(12),
                                text(format!("GHS {}", compute_accrued_amount(student)))
                            ]
                            .spacing(5)
                        ),
                    ]
                    .spacing(10),
                ]
                .spacing(40);

                let action_section = container(
                    row![
                        button(
                            container(
                                row![
                                    svg::Svg::new(icons::plus().clone())
                                        .width(16)
                                        .height(18)
                                        .style(|theme: &Theme, _status: svg::Status| {
                                            let palette = theme.extended_palette();
                                            svg::Style {
                                                color: Some(Color::WHITE),
                                                // color: Some(palette.background.weak.text),
                                            }
                                        }),
                                    text("Add Session")
                                        .size(12)
                                        .font(Font {
                                            weight: font::Weight::Semibold,
                                            ..Default::default()
                                        })
                                        .style(|theme: &Theme| {
                                            let palette = theme.extended_palette();
                                            text::Style {
                                                color: Some(Color::WHITE),
                                                // color: Some(palette.background.weak.text),
                                            }
                                        }),
                                ]
                                .spacing(5)
                                .align_y(Center)
                            )
                            .align_x(Center)
                        )
                        .style(|theme: &Theme, _status: button::Status| {
                            let palette = theme.extended_palette();
                            button::Style {
                                background: Some(Background::Color(Color::BLACK)),
                                // background: Some(Background::Color(palette.primary.base.color)),
                                border: Border {
                                    radius: 10.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        })
                        .padding(10)
                        .width(Length::FillPortion(2))
                        .height(Length::Fixed(40.0)),
                        button(
                            container(
                                row![
                                    svg::Svg::new(icons::edit().clone())
                                        .width(16)
                                        .height(18)
                                        .width(16)
                                        .height(18)
                                        .style(|theme: &Theme, _status: svg::Status| {
                                            let palette = theme.extended_palette();
                                            svg::Style {
                                                color: Some(palette.background.weak.text),
                                            }
                                        }),
                                    text("Edit").size(12).font(Font {
                                        weight: font::Weight::Semibold,
                                        ..Default::default()
                                    })
                                ]
                                .spacing(5)
                                .align_y(Center)
                            )
                            .align_x(Center)
                        )
                        .style(|theme: &Theme, _status: button::Status| {
                            let palette = theme.extended_palette();
                            button::Style {
                                background: Some(Background::Color(palette.background.weak.color)),
                                border: Border {
                                    radius: 10.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        })
                        .padding(10)
                        .width(Length::FillPortion(1))
                        .height(Length::Fixed(40.0)),
                    ]
                    .spacing(10),
                )
                .height(Length::Fixed(150.0))
                .width(Length::Fill)
                .align_y(Alignment::Start);

                let card = container(
                    column![
                        title_section,
                        column![main_section, action_section].spacing(30),
                    ]
                    .spacing(20),
                )
                .width(Length::Fixed(300.0))
                .height(Length::Fixed(500.0))
                .padding([10, 20])
                .style(move |theme: &Theme| {
                    let palette = theme.extended_palette();

                    container::Style {
                        border: Border {
                            color: palette.background.strong.color,
                            width: 1.5,
                            radius: 10.0.into(),
                            ..Default::default()
                        },
                        shadow: if is_hovered {
                            Shadow {
                                color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                                offset: Vector::new(0.4, 0.0),
                                blur_radius: 12.0,
                            }
                        } else {
                            Shadow::default()
                        },
                        ..Default::default()
                    }
                });

                mouse_area(card)
                    .interaction(Interaction::Pointer)
                    .on_enter(Message::StudentCardHovered(Some(index)))
                    .on_exit(Message::StudentCardHovered(None))
                    .into()
            })
            .collect();

        card_list
    }

    // fn view_trend_chart
}

// =========================================
// REUSABLE UI components
// =========================================
fn modal<'a, Message: 'a>(
    bg_content: Element<'a, Message>,
    modal_content: impl FnOnce() -> Element<'a, Message>,
) -> Element<'a, Message> {
    let modal_box = container(modal_content())
        .width(Length::Fill)
        .height(Length::Fill);

    let overlay = container(modal_box)
        .width(Length::Fixed(400.0))
        .height(Length::Fixed(500.0))
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(iced::Color::from_rgba(
                0.0, 0.0, 0.0, 0.5,
            ))),
            ..Default::default()
        });

    let modal = container(overlay)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(Center)
        .align_y(Center);

    stack![bg_content, modal].into()
}

#[derive(Clone, Copy)]
enum DashboardCardVariant {
    Attendance,
    ActualEarnings,
    PotentialEarnings,
    RevenueLost,
}

fn metric_card<'a>(
    title: &'a str,
    value: &'a str,
    trend: Option<(&'a str, bool)>,
    is_hovered: bool,
    card_index: Option<usize>,
    variant: DashboardCardVariant,
) -> Element<'a, Message> {
    let mut content = column![
        text(title).size(15).font(Font {
            weight: font::Weight::Medium,
            ..Default::default()
        }),
        text(value).size(25).font(Font {
            weight: font::Weight::Medium,
            ..Default::default()
        }),
    ]
    .align_x(Center)
    .spacing(5);

    if let Some((trend_text, is_positive)) = trend {
        let trend_icon = if is_positive {
            icons::arrow_up()
        } else {
            icons::arrow_down()
        };

        let trend_row = container(row![
            svg::Svg::new(trend_icon).width(14).height(14),
            text(trend_text).size(12).font(Font {
                weight: font::Weight::Medium,
                ..Default::default()
            }),
        ])
        .align_bottom(Length::Fill);

        content = content.push(trend_row);
    }

    let card = container(content)
        .height(Length::Fixed(100.0))
        .padding([10, 20])
        .center_x(Length::Fixed(180.0))
        .style(move |theme: &Theme| styles::card_style_with_variant(theme, is_hovered, variant));

    mouse_area(card)
        .on_enter(Message::DashboardCardHovered(card_index))
        .on_exit(Message::DashboardCardHovered(None))
        .into()
}

fn menu_item_container(
    item: svg::Svg,
    item_text: String,
    is_item_selected: bool,
    is_item_hovered: bool,
    is_side_menu_hovered: bool,
    container_height: f32,
) -> Container<Message> {
    let content = if is_item_hovered {
        let text_item_widget = text(format!("{}", item_text))
            .font(Font {
                weight: font::Weight::Light,
                ..Default::default()
            })
            .size(11)
            .style(|_theme: &Theme| text::Style {
                color: Some(Color {
                    r: 0.1,
                    g: 0.1,
                    b: 1.0,
                    a: 0.9,
                }),
            });
        column![item, text_item_widget].align_x(Center).spacing(5)
    } else if is_side_menu_hovered {
        let text_item_widget = text(format!("{}", item_text))
            .font(Font {
                weight: font::Weight::Light,
                ..Default::default()
            })
            .size(11);
        column![item, text_item_widget].align_x(Center).spacing(5)
    } else {
        column![item,].spacing(5)
    };

    container(content)
        .center_x(Length::Fill)
        .center_y(Length::Fixed(container_height))
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


// =========================================
// CUSTOM COMPONENTS
// =========================================
struct IncomeData {
    potential: f32,
    actual: f32,
}

struct GroupedBarChart {
    data: Vec<IncomeData>,
    cache: canvas::Cache,
}

impl GroupedBarChart {
    fn new(data: Vec<IncomeData>) -> Self {
        Self {
            data,
            cache: canvas::Cache::new(),
        }
    }
}

// impl<Message> canvas::Program<Message> for GroupedBarChart {
//     type State = ();
//     
//     fn draw(
//             &self,
//             _state: &Self::State,
//             renderer: &Renderer,
//             _theme: &Theme,
//             bounds: Rectangle,
//             _cursor: iced::advanced::mouse::Cursor,
//         ) -> Vec<canvas::Geometry<Renderer>> {
//         
//     }
// }

// =========================================
// STYLES
// =========================================

mod styles {
    use super::Background;
    use super::Border;
    use super::Color;
    use super::DashboardCardVariant;
    use super::Shadow;
    use super::Theme;
    use super::Vector;
    use super::container;
    use super::svg;

    pub fn menu_icon_style(is_item_hovered: bool) -> svg::Style {
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

    pub fn card_style_with_variant(
        theme: &Theme,
        is_hovered: bool,
        variant: DashboardCardVariant,
    ) -> container::Style {
        let palette = theme.extended_palette();

        let background_color = match variant {
            DashboardCardVariant::Attendance => Some(palette.primary.weak.color),
            DashboardCardVariant::ActualEarnings => Some(Color::from_rgba(0.4, 1.0, 0.5, 0.6)),
            DashboardCardVariant::PotentialEarnings => Some(Color::from_rgba(0.8, 0.7, 0.8, 0.4)),
            DashboardCardVariant::RevenueLost => Some(Color::from_rgba(1.0, 0.5, 0.2, 0.6)),
        };

        container::Style {
            background: background_color.map(Background::Color),
            border: Border {
                radius: 10.0.into(),
                ..Default::default()
            },
            shadow: if is_hovered {
                Shadow {
                    color: Color::from_rgba(0.0, 0.0, 0.0, 0.25),
                    offset: Vector::new(0.4, 0.0),
                    blur_radius: 12.0,
                }
            } else {
                Shadow::default()
            },
            ..Default::default()
        }
    }
}

// =========================================
// ICONS & ASSETS
// =========================================
mod icons {
    use iced::widget::svg;
    use std::sync::OnceLock;

    static PLUS: OnceLock<svg::Handle> = OnceLock::new();
    static EDIT: OnceLock<svg::Handle> = OnceLock::new();
    static CALENDAR: OnceLock<svg::Handle> = OnceLock::new();
    static SCHEDULE: OnceLock<svg::Handle> = OnceLock::new();
    static CHECK_CIRCLE: OnceLock<svg::Handle> = OnceLock::new();
    static PAYMENTS: OnceLock<svg::Handle> = OnceLock::new();
    static DASHBOARD: OnceLock<svg::Handle> = OnceLock::new();
    static ARROW_DOWN: OnceLock<svg::Handle> = OnceLock::new();
    static ARROW_UP: OnceLock<svg::Handle> = OnceLock::new();
    static STUDENT: OnceLock<svg::Handle> = OnceLock::new();

    fn icon_path(name: &str) -> String {
        format!("{}/resources/icons/{}", env!("CARGO_MANIFEST_DIR"), name)
    }

    pub fn plus() -> svg::Handle {
        PLUS.get_or_init(|| {
            svg::Handle::from_path(icon_path("add_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg"))
        })
        .clone()
    }

    pub fn edit() -> svg::Handle {
        EDIT.get_or_init(|| svg::Handle::from_path(icon_path("pen-to-square-regular-full.svg")))
            .clone()
    }

    pub fn calendar() -> svg::Handle {
        CALENDAR
            .get_or_init(|| {
                svg::Handle::from_path(icon_path(
                    "calendar_today_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg",
                ))
            })
            .clone()
    }

    pub fn schedule() -> svg::Handle {
        SCHEDULE
            .get_or_init(|| {
                svg::Handle::from_path(icon_path(
                    "schedule_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg",
                ))
            })
            .clone()
    }

    pub fn check_circle() -> svg::Handle {
        CHECK_CIRCLE
            .get_or_init(|| {
                svg::Handle::from_path(icon_path(
                    "check_circle_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg",
                ))
            })
            .clone()
    }

    pub fn payments() -> svg::Handle {
        PAYMENTS
            .get_or_init(|| {
                svg::Handle::from_path(icon_path(
                    "payments_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg",
                ))
            })
            .clone()
    }

    pub fn dashboard() -> svg::Handle {
        DASHBOARD
            .get_or_init(|| {
                svg::Handle::from_path(icon_path(
                    "dashboard_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg",
                ))
            })
            .clone()
    }

    pub fn student_manager() -> svg::Handle {
        STUDENT
            .get_or_init(|| {
                svg::Handle::from_path(icon_path(
                    "school_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg",
                ))
            })
            .clone()
    }

    pub fn arrow_up() -> svg::Handle {
        ARROW_UP
            .get_or_init(|| {
                svg::Handle::from_path(icon_path(
                    "arrow_upward_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg",
                ))
            })
            .clone()
    }

    pub fn arrow_down() -> svg::Handle {
        ARROW_DOWN
            .get_or_init(|| {
                svg::Handle::from_path(icon_path(
                    "arrow_downward_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg",
                ))
            })
            .clone()
    }
}

// =========================================
// MOCK DATA & TESTING
// =========================================
#[cfg(debug_assertions)]
fn mock_data() -> Vec<Student> {
    vec![
        Student {
            name: PersonalName {
                first: String::from("Mary"),
                last: String::from("Jane"),
                other: None,
            },
            subject: TutorSubject::AdditionalMathematics,
            tabled_sessions: vec![
                SessionData {
                    day: Weekday::Tue,
                    time: String::from("5:30 PM"),
                },
                SessionData {
                    day: Weekday::Thu,
                    time: String::from("5:30 PM"),
                },
            ],
            actual_sessions: vec![
                Local.with_ymd_and_hms(2025, 11, 4, 17, 30, 0).unwrap(),
                Local.with_ymd_and_hms(2025, 11, 6, 13, 30, 0).unwrap(),
            ],
            payment_data: PaymentData {
                per_session_amount: 150.0,
            },
        },
        Student {
            name: PersonalName {
                first: String::from("Peter"),
                last: String::from("Parker"),
                other: None,
            },
            subject: TutorSubject::ExtendedMathematics,
            tabled_sessions: vec![
                SessionData {
                    day: Weekday::Wed,
                    time: String::from("4:00 PM"),
                },
                SessionData {
                    day: Weekday::Sat,
                    time: String::from("1:30 PM"),
                },
            ],
            actual_sessions: vec![
                Local.with_ymd_and_hms(2025, 11, 5, 16, 0, 0).unwrap(),
                Local.with_ymd_and_hms(2025, 11, 8, 13, 30, 0).unwrap(),
                Local.with_ymd_and_hms(2025, 11, 22, 13, 30, 0).unwrap(),
            ],
            payment_data: PaymentData {
                per_session_amount: 150.0,
            },
        },
    ]
}

// =========================================
// MAIN
// =========================================
fn main() -> iced::Result {
    iced::application(
        TutoringManager::title,
        TutoringManager::update,
        TutoringManager::view,
    )
    .run_with(TutoringManager::new)
}
