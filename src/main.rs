use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, TimeZone, Weekday};
use iced::advanced::graphics::core::font;
use iced::mouse::Interaction;
use iced::widget::{
    Column, Container, Row, button, column, container, focus_next, mouse_area, row, stack, svg,
    text, text_input,
};
use iced::{
    Alignment, Background, Border, Center, Color, Element, Font, Length, Shadow, Task, Theme,
    Vector,
};

struct TutoringManager {
    current_screen: Screen,
    state: State,
}

struct State {
    // Main app
    selected_menu_item: SideMenuItem,
    hovered_menu_item: Option<SideMenuItem>,
    side_menu_width: f32,
    menu_item_container_height: f32,
    side_menu_hovered: bool,
    show_menu_text: bool,

    // Dashboard
    dashboard_card_hovered: bool,

    // StudentManager
    search_query: String,
    show_add_student_modal: bool,
    students: Vec<Student>,
    hovered_student_card: Option<usize>,
}

impl TutoringManager {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                current_screen: Screen::Dashboard,
                state: State {
                    selected_menu_item: SideMenuItem::Dashboard,
                    hovered_menu_item: None,
                    side_menu_width: 50.0,
                    side_menu_hovered: false,
                    show_menu_text: false,
                    menu_item_container_height: 50.0,

                    dashboard_card_hovered: false,

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
            Message::NavigateToScreen(menu_item) => {
                self.state.selected_menu_item = menu_item.clone();
                self.current_screen = match menu_item {
                    SideMenuItem::Dashboard => Screen::Dashboard,
                    SideMenuItem::StudentManager => Screen::StudentManager,
                };
                Task::none()
            }
            Message::DashboardCardHovered(is_hovered) => {
                self.state.dashboard_card_hovered = is_hovered;
                Task::none()
            }
            Message::MenuItemHovered(menu_item_opt) => {
                self.state.hovered_menu_item = menu_item_opt.clone();
                self.state.side_menu_width = match menu_item_opt {
                    Some(SideMenuItem::Dashboard) => 90.0,
                    Some(SideMenuItem::StudentManager) => 90.0,
                    None => 50.0,
                };
                self.state.menu_item_container_height = match menu_item_opt {
                    Some(SideMenuItem::Dashboard) => 90.0,
                    Some(SideMenuItem::StudentManager) => 90.0,
                    None => 50.0,
                };
                Task::none()
            }
            Message::SideMenuHovered(is_hovered) => {
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
            Message::ShowAddStudentModal => {
                self.state.show_add_student_modal = true;
                focus_next()
            }
            Message::CloseAddStudentModal => {
                self.state.show_add_student_modal = false;
                Task::none()
            }
            Message::StudentCardHovered(card_index) => {
                self.state.hovered_student_card = card_index;
                Task::none()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
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

        let dash_handle = svg::Handle::from_path(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/icons/dashboard_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg"
        ));
        let dash_icon = svg::Svg::new(dash_handle.clone())
            .width(25)
            .height(25)
            .style(move |_theme: &Theme, _status: svg::Status| menu_icon_style(is_dash_hovered));
        let dash_item_text = String::from("Dashboard");

        let student_handle = svg::Handle::from_path(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/icons/school_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg"
        ));
        let student_icon = svg::Svg::new(student_handle.clone())
            .width(25)
            .height(25)
            .style(move |_theme: &Theme, _status: svg::Status| menu_icon_style(is_student_hovered));
        let student_item_text = String::from("Student Manager");

        let plus_handle = svg::Handle::from_path(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/icons/add_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg"
        ));
        let plus_icon = svg::Svg::new(plus_handle.clone()).width(25).height(25);
        let edit_handle = svg::Handle::from_path(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/icons/pen-to-square-regular-full.svg"
        ));

        let calendar_handle = svg::Handle::from_path(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/icons/calendar_today_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg"
        ));

        let schedule_handle = svg::Handle::from_path(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/icons/schedule_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg"
        ));

        let complete_handle = svg::Handle::from_path(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/icons/check_circle_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg"
        ));

        let payments_handle = svg::Handle::from_path(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/icons/payments_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg"
        ));

        let side_menu = mouse_area(
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
        .on_exit(Message::SideMenuHovered(false));

        let dashboard_page: Element<Message> = {
            let page_title_text = text!("Dashboard").size(20);
            let page_title = row![page_title_text];

            let attendance_rate_card = mouse_area(
                container(
                    column![
                        text("Attendance Rate").size(15).font(Font {
                            weight: font::Weight::Medium,
                            ..Default::default()
                        }),
                        text("85%").size(25).font(Font {
                            weight: font::Weight::Medium,
                            ..Default::default()
                        }),
                        container(text("5% MoM").size(12).font(Font {
                            weight: font::Weight::Light,
                            ..Default::default()
                        }))
                        .align_bottom(Length::Fill),
                    ]
                    .align_x(Center)
                    .spacing(5),
                )
                .height(Length::Fixed(100.0))
                .padding([10, 20])
                .center_x(Length::Fixed(180.0))
                .style(move |theme: &Theme| {
                    let palette = theme.extended_palette();

                    container::Style {
                        border: Border {
                            color: palette.background.strong.color,
                            width: 1.5,
                            radius: 10.0.into(),
                            ..Default::default()
                        },
                        shadow: if self.state.dashboard_card_hovered {
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
                }),
            )
            .on_enter(Message::DashboardCardHovered(true))
            .on_exit(Message::DashboardCardHovered(false));

            let actual_earnings_card = column![text("Actual Earnings"), text("GHS 1500"),];

            let potential_earnings_card = column![text("Potential Earnings"), text("GHS 2000"),];

            let revenue_lost_card = column![text("Revenue Lost"), text("GHS 2000"),];

            let summary_cards = row![
                attendance_rate_card,
                actual_earnings_card,
                potential_earnings_card,
                revenue_lost_card,
            ]
            .spacing(10);

            let attendance_trend_chart = container(text("Trend chart"));
            let potential_vs_actual_chart = container(text("Bar chart"));

            let graphs = row![attendance_trend_chart, potential_vs_actual_chart,];

            container(column![page_title, summary_cards, graphs].spacing(30))
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        };

        let student_page: Element<Message> = {
            let page_title_text = text!("Student Manager")
                .font(Font {
                    weight: font::Weight::Bold,
                    ..Default::default()
                })
                .size(24);
            let page_title = row![page_title_text];

            let search_bar = container(text_input("Search students", &self.state.search_query));
            let add_button = button(plus_icon).on_press(Message::ShowAddStudentModal);
            let action_bar = row![search_bar, add_button].spacing(100);
            let card_container = {
                let card_contents =
                    self.state
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
                                        text(format!(
                                            "{} {}",
                                            student.name.first, student.name.last
                                        ))
                                        .font(Font {
                                            weight: font::Weight::Bold,
                                            ..Default::default()
                                        })
                                        .size(20)
                                    } else {
                                        text(format!(
                                            "{} {} {}",
                                            student.name.first,
                                            student_other_name,
                                            student.name.last
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
                                        svg::Svg::new(calendar_handle.clone()).width(22).height(22)
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
                                                .extend(student.tabled_sessions.iter().map(
                                                    |session| {
                                                        text(format!(
                                                            "{} {}",
                                                            session.day.to_string(),
                                                            session.time
                                                        ))
                                                        .into()
                                                    }
                                                ))
                                                .spacing(2),
                                        ]
                                        .spacing(4)
                                    ),
                                ]
                                .spacing(10),
                                row![
                                    container(
                                        svg::Svg::new(schedule_handle.clone()).width(22).height(22)
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
                                        svg::Svg::new(complete_handle.clone()).width(22).height(22)
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
                                            text(format!(
                                                "{}",
                                                compute_num_of_completed_sessions(student)
                                            )),
                                        ]
                                        .spacing(5),
                                    ),
                                ]
                                .spacing(10),
                                row![
                                    container(
                                        svg::Svg::new(payments_handle.clone()).width(22).height(22)
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
                                            text(format!(
                                                "GHS {}",
                                                compute_accrued_amount(student)
                                            ))
                                        ]
                                        .spacing(5)
                                    ),
                                ]
                                .spacing(10),
                            ]
                            .spacing(40);

                            let action_section =
                                container(
                                    row![
                                        button(
                                            container(
                                                row![
                                        svg::Svg::new(plus_handle.clone())
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
                                        svg::Svg::new(edit_handle.clone())
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
                                                background: Some(Background::Color(
                                                    palette.background.weak.color,
                                                )),
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
                        });

                container(Row::new().extend(card_contents).spacing(30))
            };

            let main_container =
                container(column![page_title, action_bar, card_container].spacing(30))
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
        };

        let main_area = {
            match self.current_screen {
                Screen::Dashboard => container(dashboard_page).padding(20),
                Screen::StudentManager => container(student_page).padding(20),
            }
        };

        container(row![side_menu, main_area].spacing(20)).into()
    }
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

fn main() -> iced::Result {
    iced::application(
        TutoringManager::title,
        TutoringManager::update,
        TutoringManager::view,
    )
    .run_with(TutoringManager::new)
}

#[derive(Debug, Clone)]
enum Message {
    // Main app
    NavigateToScreen(SideMenuItem),
    MenuItemHovered(Option<SideMenuItem>),
    SideMenuHovered(bool),

    // Dashboard
    DashboardCardHovered(bool),

    // Student Manager
    ShowAddStudentModal,
    CloseAddStudentModal,
    StudentCardHovered(Option<usize>),
}

// CUSTOM COMPONENTS
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

// DOMAIN MODELS
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

// STYLE FUNCTIONS
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

// MOCK DATA
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
