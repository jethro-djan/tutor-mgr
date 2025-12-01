use chrono::{DateTime, Datelike, Duration, Local, NaiveDate, TimeZone, Weekday};
use iced::advanced::layout::contained;
use iced::mouse::Interaction;
use iced::widget::{
    Column, Row, button, column, container, focus_next, horizontal_rule, mouse_area, row, stack,
    svg, text, text_input, Container,
};
use iced::{Background, Border, Center, Color, Element, Length, Renderer, Task, Theme};

struct TutoringManager {
    current_screen: Screen,
    state: State,
}

struct State {
    // Main app
    selected_menu_item: SideMenuItem,

    // Dashboard

    // StudentManager
    search_query: String,
    show_add_student_modal: bool,
    students: Vec<Student>,
}

impl TutoringManager {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                current_screen: Screen::Dashboard,
                state: State {
                    selected_menu_item: SideMenuItem::Dashboard,
                    search_query: String::new(),
                    show_add_student_modal: false,
                    students: mock_data(),
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
                self.current_screen = match menu_item {
                    SideMenuItem::Dashboard => Screen::Dashboard,
                    SideMenuItem::StudentManager => Screen::StudentManager,
                };
                Task::none()
            }
            Message::MenuItemSelected(menu_item) => {
                self.state.selected_menu_item = menu_item;
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
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let is_dash_selected = match self.state.selected_menu_item {
            SideMenuItem::Dashboard => true,
            SideMenuItem::StudentManager => false
        };
        let is_student_selected = match self.state.selected_menu_item {
            SideMenuItem::Dashboard => false,
            SideMenuItem::StudentManager => true,
        };
        let dash_handle = svg::Handle::from_path(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/icons/dashboard_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg"
        ));
        let dash_icon = svg::Svg::new(dash_handle.clone())
            .width(25)
            .height(25)
            .style(menu_icon_style);

        let student_handle = svg::Handle::from_path(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/icons/school_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg"
        ));
        let student_icon = svg::Svg::new(student_handle.clone())
            .width(25)
            .height(25)
            .style(menu_icon_style);
        let plus_handle = svg::Handle::from_path(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/icons/add_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg"
        ));
        let plus_icon = svg::Svg::new(plus_handle.clone()).width(25).height(25);
        let edit_handle = svg::Handle::from_path(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/icons/pen-to-square-regular-full.svg"
        ));

        let side_menu = container(
            column![
                mouse_area(menu_item_container(dash_icon, is_dash_selected))
                    .interaction(Interaction::Pointer)
                    .on_press(Message::NavigateToScreen(SideMenuItem::Dashboard))
                    .on_enter(Message::MenuItemSelected(SideMenuItem::Dashboard)),
                mouse_area(menu_item_container(student_icon, is_student_selected))
                    .interaction(Interaction::Pointer)
                    .on_press(Message::NavigateToScreen(SideMenuItem::StudentManager))
                    .on_enter(Message::MenuItemSelected(SideMenuItem::StudentManager)),
            ]
            .spacing(20),
        )
        .padding([250, 0])
        .width(70)
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
        });

        let student_page: Element<Message> = {
            let page_title_text = text!("Student Manager").size(20);
            let page_title = row![page_title_text];

            let search_bar = container(text_input("Search students", &self.state.search_query));
            let add_button = button(plus_icon).on_press(Message::ShowAddStudentModal);
            let action_bar = row![search_bar, add_button].spacing(100);
            let card_container = {
                let students = self.state.students.iter().map(|student| {
                    let next_session = get_next_session(student);
                    let day = next_session.format("%A").to_string();
                    let date = next_session.format("%d %B %Y").to_string();

                    container(column![
                        text(format!("{} {}", student.name.first, student.name.last)),
                        horizontal_rule(1),
                        row![
                            text("Subject(s):").width(Length::Fixed(120.0)),
                            text(student.subject.as_str()),
                        ]
                        .spacing(60),
                        row![
                            text("Schedule:").width(Length::Fixed(120.0)),
                            Column::new()
                                .extend(student.tabled_sessions.iter().map(|session| {
                                    text(format!("{} {}", session.day.to_string(), session.time))
                                        .into()
                                }))
                                .spacing(2),
                        ]
                        .spacing(60),
                        row![
                            text("Next session:").width(Length::Fixed(120.0)),
                            text(format!("{}, {}", day, date))
                        ]
                        .spacing(60),
                        row![
                            text("Completed sessions:").width(Length::Fixed(120.0)),
                            text(format!("{}", compute_num_of_completed_sessions(student))),
                        ]
                        .spacing(60),
                        row![
                            text("Amount accrued:").width(Length::Fixed(120.0)),
                            text(format!("GHS {}", compute_accrued_amount(student)))
                        ]
                        .spacing(60),
                        container(
                            row![
                                container(mouse_area(row![
                                    svg::Svg::new(plus_handle.clone()).width(20).height(20),
                                    text("Add session").size(14)
                                ]))
                                .align_left(Length::Fill),
                                container(mouse_area(row![
                                    svg::Svg::new(edit_handle.clone()).width(20).height(20),
                                    text("Edit").size(14)
                                ]))
                                .align_right(Length::Fill)
                            ]
                            .spacing(50),
                        )
                        .align_bottom(Length::Fill)
                    ])
                    .width(Length::Fixed(400.0))
                    .height(Length::Fixed(250.0))
                    .padding([10, 20])
                    .style(|theme: &Theme| {
                        let palette = theme.extended_palette();

                        container::Style {
                            border: Border {
                                color: palette.background.strong.color,
                                width: 1.5,
                                radius: 10.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }
                    })
                    .into()
                });

                container(Row::new().extend(students).spacing(10))
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
                Screen::Dashboard => container(text!("Dashboard")).padding(20),
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
    MenuItemSelected(SideMenuItem),

    // Student Manager
    ShowAddStudentModal,
    CloseAddStudentModal,
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
fn menu_icon_style(theme: &Theme, status: svg::Status) -> svg::Style {
    match status {
        svg::Status::Idle => svg::Style { color: None },
        svg::Status::Hovered => svg::Style {
            color: Some(Color {
                r: 0.3,
                g: 0.6,
                b: 1.0,
                a: 0.5,
            }),
        },
    }
}

fn menu_item_container(item: svg::Svg, is_item_selected: bool) -> Container<Message> {
    container(item)
        .center_x(Length::Fill)
        .center_y(Length::Fixed(50.0))
        .style(move |theme: &Theme| 
            if is_item_selected {
                container::Style {
                    text_color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.5)), 
                    background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.5))), 
                    ..Default::default()
                }
            } else {
                container::transparent(theme)
            }
        )
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
