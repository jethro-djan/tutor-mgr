use iced::{Task, Element, Length, Theme, Border, border, Color};
use iced::widget::{self, text, container, column, row, button, mouse_area, svg, text_input};

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
            Task::none()
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
                widget::focus_next()
            }
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let dash_icon = 
            svg(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/icons/dashboard_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg"))
                .width(25)
                .height(25);
        let student_icon = 
            svg(concat!(env!("CARGO_MANIFEST_DIR"), "/resources/icons/school_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg"))
                .width(25)
                .height(25);
        let side_menu = 
            container(
                column![
                    mouse_area(dash_icon).on_press(Message::NavigateToScreen(SideMenuItem::Dashboard)),
                    mouse_area(student_icon).on_press(Message::NavigateToScreen(SideMenuItem::StudentManager)),
                ]
                .spacing(20)
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

        let student_page =  {
            let page_title_text = text!("Student Manager")
                .size(20);
            let page_title = row![page_title_text];

            let search_bar = text_input("Search students", &self.state.search_query);
            let add_button = button(text!("add")).on_press(Message::ShowAddStudentModal);

            let action_bar = 
                row![search_bar, add_button]
                    .spacing(20);
            if self.state.show_add_student_modal {
                let modal = container(
                    column![
                        row![text!("Modal open")],
                    ]
                );

                modal
            } else {
                container(
                    column![page_title, action_bar]
                        .spacing(15)
                )
            }
        };

        let main_area = {
            match self.current_screen {
                Screen::Dashboard => {
                    container(text!("Dashboard"))
                        .padding(20)
                }
                Screen::StudentManager => {
                    container(student_page)
                        .padding(20)
                }
            }
        };

        container(
            row![
                side_menu, 
                main_area
            ]
            .spacing(20)
        )
        .into()
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
        TutoringManager::view
    )
    .run_with(TutoringManager::new)
}

#[derive(Debug, Clone)]
enum Message {
    NavigateToScreen(SideMenuItem),

    // Student Manager
    ShowAddStudentModal,
}

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
