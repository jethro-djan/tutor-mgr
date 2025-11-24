use iced::{Task, Element, Length, Theme, Border, border, Color};
use iced::widget::{text, container, column, row, button, mouse_area, svg};

struct TutoringManager {
    current_screen: Screen,
}

impl TutoringManager {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                current_screen: Screen::Dashboard,
            },
            Task::none()
        )
    }

    fn title(&self) -> String {
        String::from("Tutor Manager")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::NavigateToScreen(menu_item) => {
                self.current_screen = match menu_item {
                    SideMenuItem::Dashboard => Screen::Dashboard,
                    SideMenuItem::StudentManager => Screen::StudentManager,
                }
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
                .spacing(15)
            )
            .padding(20)
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
        let main_area = {
            match self.current_screen {
                Screen::Dashboard => {
                    container(text!("Dashboard"))
                }
                Screen::StudentManager => {
                    container(text!("Student manager"))
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
