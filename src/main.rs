use iced::widget::{
    button, column, container, focus_next, horizontal_rule, mouse_area, row, stack, svg, text,
    text_input,
};
use iced::{Border, Center, Element, Length, Task, Theme};

struct TutoringManager {
    current_screen: Screen,
    state: State,
}

struct State {
    // Dashboard

    // StudentManager
    search_query: String,
    show_add_student_modal: bool,
}

impl TutoringManager {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                current_screen: Screen::Dashboard,
                state: State {
                    search_query: String::new(),
                    show_add_student_modal: false,
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
        let dash_icon = svg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/icons/dashboard_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg"
        ))
        .width(25)
        .height(25);
        let student_icon = svg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/icons/school_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg"
        ))
        .width(25)
        .height(25);
        let plus_handle = svg::Handle::from_path(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/icons/add_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg"
        ));
        let plus_icon = svg::Svg::new(plus_handle.clone()).width(25).height(25);
        let edit_handle = svg::Handle::from_path(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/resources/icons/pen-to-square-regular-full.svg"
        ));
        let edit_icon = svg::Svg::new(edit_handle.clone()).width(20).height(20);

        let side_menu = container(
            column![
                mouse_area(dash_icon).on_press(Message::NavigateToScreen(SideMenuItem::Dashboard)),
                mouse_area(student_icon)
                    .on_press(Message::NavigateToScreen(SideMenuItem::StudentManager)),
            ]
            .spacing(20),
        )
        .padding([250, 20])
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
                let student1 = container(column![
                    text!("Mary Jane"),
                    horizontal_rule(1),
                    row![
                        text("Subject(s):").width(Length::Fixed(120.0)),
                        text!("Extended Mathematics")
                    ]
                    .spacing(60),
                    row![
                        text("Schedule:").width(Length::Fixed(120.0)),
                        column![text!("Tue 5:30 PM"), text!("Thu 5:30 PM")].spacing(2)
                    ]
                    .spacing(60),
                    row![
                        text("Next session:").width(Length::Fixed(120.0)),
                        text!("Friday, Nov 18")
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
                .height(Length::Fixed(200.0))
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
                });

                let student2 = container(column![
                    text!("Peter Parker"),
                    horizontal_rule(1),
                    row![
                        text("Subject(s):").width(Length::Fixed(120.0)),
                        text!("Additional Mathematics")
                    ]
                    .spacing(60),
                    row![
                        text("Schedule:").width(Length::Fixed(120.0)),
                        column![text!("Tue 5:30 PM"), text!("Thu 5:30 PM")].spacing(2)
                    ]
                    .spacing(60),
                    row![
                        text("Next session:").width(Length::Fixed(120.0)),
                        text!("Thursday, Nov 17")
                    ]
                    .spacing(60),
                    container(
                        row![
                            container(mouse_area(row![
                                svg::Svg::new(plus_handle.clone()).width(20).height(20),
                                text("Add session").size(14)
                            ]))
                            .align_left(Length::Fill),
                            container(mouse_area(row![edit_icon, text("Edit").size(14)]))
                                .align_right(Length::Fill)
                        ]
                        .spacing(50),
                    )
                    .align_bottom(Length::Fill)
                ])
                .width(Length::Fixed(400.0))
                .height(Length::Fixed(200.0))
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
                });

                container(row![student1, student2].spacing(10))
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
    NavigateToScreen(SideMenuItem),

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
    session_data: SessionData,

    payment_data: PaymentData,
}

struct PersonalName {
    first: String,
    last: String,
    other: Option<String>,
}

struct SessionData {
    days: Vec<MeetingDays>,
    subject: TutorSubjects,
    time: String,
}

enum MeetingDays {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

struct Session {
    date: String,
    time: String,
    subject: TutorSubjects,
}

enum TutorSubjects {
    AdditionalMathematics,
    ExtendedMathematics,
    Statistics,
}

struct PaymentData {
    per_session_amount: f32,
    due_date: String,
}
