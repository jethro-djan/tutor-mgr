use chrono::{Datelike, Local, Weekday};
use iced::advanced::graphics::core::font;
use iced::mouse::Interaction;
use iced::widget::PickList;
use iced::widget::{
    Column, Container, Row, Stack, button, center, column, container, mouse_area, opaque,
    operation::focus_next, pick_list, row, space, stack, svg, text, text_input,
};
use iced::{
    Alignment, Background, Border, Center, Color, Element, Font, Length, Padding, Shadow, Task,
    Theme, Vector,
};
use std::rc::Rc;

use crate::domain::{
    Domain, SessionData, Student, Tutor, TutorSubject, compute_monthly_completed_sessions,
    compute_monthly_sum, get_next_session,
};
use crate::icons;
use crate::ui_components::{global_content_container, page_header, ui_button};

#[derive(Clone, Debug)]
pub struct TimeSlot {
    pub id: usize,
    pub selected_day: Option<DaySelection>,
    pub selected_time: Option<TimeSelection>,
}

pub struct StudentManagerState {
    pub search_query: String,
    pub show_add_student_modal: bool,
    pub hovered_student_card: Option<usize>,
    pub tutor: Option<Tutor>,
    pub students: Option<Vec<Student>>,

    // Modal State
    pub modal_state: ModalState,
}

pub struct ModalState {
    pub modal_input: ModalInput,
    pub selected_subject: Option<TutorSubject>,

    pub time_slots: Vec<TimeSlot>,
    pub next_slot_id: usize,
}

impl ModalState {
    pub fn clear(&mut self) {
        self.modal_input.first_name = String::new();
        self.modal_input.last_name = String::new();
        self.modal_input.other_names = String::new();
        self.modal_input.subject = String::new();
        self.modal_input.pay_rate = String::new();
        self.modal_input.weekly_schedule = WeeklySchedule(Vec::new());
        self.selected_subject = None;
        self.time_slots = vec![TimeSlot {
            id: 0,
            selected_day: None,
            selected_time: None,
        }];
        self.next_slot_id = 1;
    }
}

impl StudentManagerState {
    pub fn attach_domain(&mut self, domain: Rc<Domain>) {
        self.search_query = String::new();
        self.show_add_student_modal = false;
        self.hovered_student_card = None;
        self.tutor = Some(domain.tutor.clone());
        self.students = Some(domain.students.clone());
        self.modal_state.modal_input = ModalInput::default();
        self.modal_state.selected_subject = None;

        self.modal_state.time_slots = vec![TimeSlot {
            id: 0,
            selected_day: None,
            selected_time: None,
        }];
        self.modal_state.next_slot_id = 1;
    }

    pub fn empty() -> Self {
        Self {
            search_query: String::new(),
            show_add_student_modal: false,
            hovered_student_card: None,
            tutor: None,
            students: None,
            modal_state: ModalState {
                modal_input: ModalInput::default(),
                selected_subject: None,
                time_slots: vec![TimeSlot {
                    id: 0,
                    selected_day: None,
                    selected_time: None,
                }],
                next_slot_id: 1,
            },
        }
    }
}

#[derive(Clone, Debug)]
pub enum Msg {
    StudentCardHovered(Option<usize>),

    ShowAddStudentModal,
    CloseAddStudentModal,
    SubjectSelected(TutorSubject),
    FirstNameInputChanged(String),
    LastNameInputChanged(String),
    OtherNamesInputChanged(String),
    RateInputChanged(String),

    AddTimeSlot,
    RemoveTimeSlot(usize),

    TutoringDaySelected(usize, DaySelection),
    TutoringTimeSelected(usize, TimeSelection),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TimeSelection {
    // None,
    Time(String),
}

impl std::fmt::Display for TimeSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // TimeSelection::None => write!(f, " "),
            TimeSelection::Time(time) => write!(f, "{}", time),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DaySelection {
    // None,
    Day(Weekday),
}

impl std::fmt::Display for DaySelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // DaySelection::None => write!(f, " "),
            DaySelection::Day(day) => write!(f, "{}", day),
        }
    }
}

#[derive(Default, Debug)]

pub struct WeeklySchedule(pub Vec<SessionData>);

#[derive(Default, Debug)]
pub struct ModalInput {
    pub first_name: String,
    pub last_name: String,
    pub other_names: String,

    pub subject: String,

    pub pay_rate: String,

    pub weekly_schedule: WeeklySchedule,
}

pub fn update(state: &mut StudentManagerState, msg: Msg) -> Task<Msg> {
    match msg {
        Msg::ShowAddStudentModal => {
            state.show_add_student_modal = true;
            focus_next()
        }
        Msg::CloseAddStudentModal => {
            state.show_add_student_modal = false;
            state.modal_state.clear();
            Task::none()
        }
        Msg::SubjectSelected(subject) => {
            state.modal_state.selected_subject = Some(subject);
            Task::none()
        }
        Msg::StudentCardHovered(card_idx_opt) => {
            state.hovered_student_card = card_idx_opt;
            Task::none()
        }

        Msg::AddTimeSlot => {
            if state.modal_state.time_slots.len() < 3 {
                state.modal_state.time_slots.push(TimeSlot {
                    id: state.modal_state.next_slot_id,
                    selected_day: None,
                    selected_time: None,
                });
            }
            state.modal_state.next_slot_id += 1;
            Task::none()
        }

        Msg::RemoveTimeSlot(id) => {
            state.modal_state.time_slots.retain(|slot| slot.id != id);
            if state.modal_state.time_slots.is_empty() {
                state.modal_state.time_slots.push(TimeSlot {
                    id: state.modal_state.next_slot_id,
                    selected_day: None,
                    selected_time: None,
                });
                state.modal_state.next_slot_id += 1;
            }
            Task::none()
        }

        Msg::TutoringDaySelected(slot_id, day) => {
            if let Some(slot) = state
                .modal_state
                .time_slots
                .iter_mut()
                .find(|s| s.id == slot_id)
            {
                slot.selected_day = Some(day);
                slot.selected_time = None;
            }
            Task::none()
        }

        Msg::TutoringTimeSelected(slot_id, time) => {
            if let Some(slot) = state
                .modal_state
                .time_slots
                .iter_mut()
                .find(|s| s.id == slot_id)
            {
                slot.selected_time = Some(time);
            }
            Task::none()
        }

        Msg::FirstNameInputChanged(name) => {
            state.modal_state.modal_input.first_name = name;
            Task::none()
        }

        Msg::LastNameInputChanged(name) => {
            state.modal_state.modal_input.last_name = name;
            Task::none()
        }

        Msg::OtherNamesInputChanged(name) => {
            state.modal_state.modal_input.other_names = name;
            Task::none()
        }

        Msg::RateInputChanged(amount) => {
            state.modal_state.modal_input.pay_rate = amount;
            Task::none()
        }
    }
}

pub fn view(state: &StudentManagerState) -> Element<'_, Msg> {
    view_student_manager(state)
}

fn view_student_manager(state: &StudentManagerState) -> Element<'_, Msg> {
    let search_bar = view_search_bar("Search Students", &state.search_query);
    let add_button = button(
        row![
            svg(icons::plus())
                .width(22)
                .height(22)
                .style(|_theme: &Theme, _status| {
                    svg::Style {
                        color: Some(Color::from_rgba(0.0, 0.2, 0.9, 0.7)),
                    }
                }),
            text("Add Student")
                .font(Font {
                    weight: font::Weight::Medium,
                    ..Default::default()
                })
                .style(|_theme: &Theme| {
                    text::Style {
                        color: Some(Color::from_rgba(0.0, 0.2, 0.9, 0.7)),
                    }
                }),
        ]
        .align_y(iced::Center)
        .spacing(5),
    )
    .style(|_theme, _status| button::Style {
        background: None,
        ..Default::default()
    })
    .on_press(Msg::ShowAddStudentModal);
    let action_bar = row![search_bar, add_button].spacing(100);
    let card_container = container(
        Row::new()
            .extend(view_student_manager_card_list(state))
            .spacing(30),
    );

    let header = page_header("Student Manager");

    let main_area_content =
        global_content_container(column![action_bar, card_container,].spacing(30))
            .width(Length::Fill)
            .height(Length::Fill);

    let main_container = column![header, main_area_content,];

    if state.show_add_student_modal {
        modal(main_container, modal_content_container(state)).into()
    } else {
        main_container.into()
    }
}

fn modal_content_container(state: &StudentManagerState) -> Element<'_, Msg> {
    let basic_info_section = column![
        container(text("Basic Information").size(18).font(Font {
            weight: font::Weight::Semibold,
            ..Default::default()
        }))
        .padding([20, 0]),
        column![
            row![
                column![
                    text("First Name").size(13).font(Font {
                        weight: font::Weight::Medium,
                        ..Default::default()
                    }),
                    text_input("John", &state.modal_state.modal_input.first_name)
                        .on_input(Msg::FirstNameInputChanged),
                ]
                .spacing(5),
                column![
                    text("Last Name").size(13).font(Font {
                        weight: font::Weight::Medium,
                        ..Default::default()
                    }),
                    text_input("Smith", &state.modal_state.modal_input.last_name)
                        .on_input(Msg::LastNameInputChanged),
                ]
                .spacing(5),
                column![
                    text("Other Names").size(13).font(Font {
                        weight: font::Weight::Medium,
                        ..Default::default()
                    }),
                    text_input("", &state.modal_state.modal_input.other_names)
                        .on_input(Msg::OtherNamesInputChanged),
                ]
                .spacing(5),
            ]
            .spacing(20),
            column![
                text("Subject").size(13).font(Font {
                    weight: font::Weight::Medium,
                    ..Default::default()
                }),
                pick_list(
                    state.tutor.as_ref().unwrap().subjects.clone(),
                    state.modal_state.selected_subject,
                    Msg::SubjectSelected
                )
                .placeholder("Pick tutor subject")
                .menu_height(100),
                // space().height(80),
            ]
            .padding([10, 0])
            .spacing(5),
            column![
                text("Rate per session (GHS)").size(13).font(Font {
                    weight: font::Weight::Medium,
                    ..Default::default()
                }),
                text_input("e.g., 150", &state.modal_state.modal_input.pay_rate)
                    .on_input(Msg::RateInputChanged),
            ]
            .spacing(5),
        ]
        .spacing(20),
    ];

    let days: Vec<DaySelection> = state
        .tutor
        .as_ref()
        .unwrap()
        .tutoring_days
        .clone()
        .into_iter()
        .map(DaySelection::Day)
        .collect();

    let schedule_section = column![
        container(row![
            text("Weekly Schedule").size(18).font(Font {
                weight: font::Weight::Semibold,
                ..Default::default()
            }),
            space().width(Length::Fill),
            mouse_area(
                ui_button(
                    "Add Time Slot",
                    12.0,
                    icons::plus(),
                    16.0,
                    18.0,
                    |_| Color::from_rgba(0.0, 0.2, 0.9, 0.7),
                    |theme| theme.extended_palette().background.weak.color,
                )
                .padding(5)
                .on_press(Msg::AddTimeSlot)
            )
            .interaction(Interaction::Pointer),
        ])
        .padding([20, 0]),
    ]
    .extend(state.modal_state.time_slots.iter().map(|slot| {
        let slot_id = slot.id;
        let can_remove = state.modal_state.time_slots.len() > 1;

        row![
            pick_list(days.clone(), slot.selected_day.clone(), move |day| {
                Msg::TutoringDaySelected(slot_id, day)
            },)
            .placeholder("Select Day")
            .width(Length::FillPortion(1))
            .menu_height(155),
            space().width(Length::Fixed(20.0)),
            {
                let time_picker: Element<Msg> =
                    if let Some(DaySelection::Day(day)) = slot.selected_day {
                        let times: Vec<TimeSelection> = state
                            .tutor
                            .as_ref()
                            .unwrap()
                            .available_times
                            .get(&day)
                            .cloned()
                            .unwrap_or_default()
                            .into_iter()
                            .map(TimeSelection::Time)
                            .collect();

                        pick_list(times.clone(), slot.selected_time.clone(), move |time| {
                            Msg::TutoringTimeSelected(slot_id, time)
                        })
                        .placeholder("Select Time")
                        .width(Length::FillPortion(1))
                        .menu_height((times.len() as f32) * 35.0)
                        .into()
                    } else {
                        pick_list(
                            Vec::<TimeSelection>::new(),
                            slot.selected_time.clone(),
                            move |time| Msg::TutoringTimeSelected(slot_id, time),
                        )
                        .placeholder("--:-- --")
                        .width(Length::FillPortion(1))
                        .menu_height(0)
                        .into()
                    };

                time_picker
            },
            space().width(Length::Fixed(10.0)),
            {
                let remove_button: Element<Msg> = if can_remove {
                    mouse_area(
                        button(svg::Svg::new(icons::delete()).style(|_theme, _status| {
                            svg::Style {
                                color: Some(Color::from_rgba(1.0, 0.0, 0.2, 1.0)),
                            }
                        }))
                        .padding(5)
                        .width(Length::Fixed(30.0))
                        .style(|theme: &Theme, _status| {
                            let palette = theme.extended_palette();
                            button::Style {
                                background: Some(Background::Color(palette.background.weak.color)),
                                ..Default::default()
                            }
                        })
                        .on_press(Msg::RemoveTimeSlot(slot_id)),
                    )
                    .interaction(Interaction::Pointer)
                    .into()
                } else {
                    space().width(Length::Fixed(30.0)).into()
                };

                remove_button
            }
        ]
        .spacing(10)
        .padding([10, 0])
        .into()
    }))
    .spacing(10);

    let action_section = container(
        row![
            mouse_area(
                ui_button(
                    "Cancel",
                    12.0,
                    icons::cancel(),
                    16.0,
                    18.0,
                    |theme| theme.extended_palette().background.weak.text,
                    |theme| theme.extended_palette().background.weak.color,
                )
                .style(|_theme, _status| {
                    button::Style {
                        border: Border {
                            color: Color::BLACK,
                            width: 1.0,
                            radius: 10.0.into(),
                        },
                        ..Default::default()
                    }
                })
                .padding(10)
                .width(Length::FillPortion(1))
                .height(Length::Fixed(40.0))
                .on_press(Msg::CloseAddStudentModal)
            )
            .interaction(Interaction::Pointer),
            mouse_area(
                ui_button(
                    "Add Student",
                    12.0,
                    icons::plus(),
                    16.0,
                    18.0,
                    |_| Color::WHITE,
                    |_| Color::BLACK,
                )
                .padding(10)
                .width(Length::FillPortion(1))
                .height(Length::Fixed(40.0))
                .on_press(Msg::CloseAddStudentModal),
            )
            .interaction(Interaction::Pointer),
        ]
        .spacing(10),
    )
    .height(Length::Fixed(100.0))
    .width(Length::Fill)
    .padding(Padding {
        top: 0.0,
        left: 0.0,
        right: 0.0,
        bottom: 20.0,
    })
    .align_y(Alignment::End);

    container(column![
        page_header("Add New Student").padding([10, 0]),
        basic_info_section,
        schedule_section,
        action_section,
    ])
    .width(600)
    .padding([10, 30])
    .style(container::rounded_box)
    .into()
}

fn view_search_bar<'a>(
    placeholder_text: &'a str,
    search_query_state_tracker: &'a str,
) -> Element<'a, Msg> {
    container(text_input(placeholder_text, &search_query_state_tracker)).into()
}

fn view_student_manager_card_list(state: &StudentManagerState) -> Vec<Element<'_, Msg>> {
    match &state.students {
        None => loading_student_cards(),
        Some(students) => render_student_cards(state, students),
    }
}

fn render_student_cards<'a>(
    state: &'a StudentManagerState,
    students: &'a Vec<Student>,
) -> Vec<Element<'a, Msg>> {
    let today = Local::now().naive_local().date();
    let current_year = today.year();
    let current_month = today.month();

    let card_list = students
        .iter()
        .enumerate()
        .map(|(index, student)| {
            let next_session = get_next_session(student);
            let day = next_session.format("%A").to_string();
            let date = next_session.format("%d %B %Y").to_string();

            let is_hovered = state.hovered_student_card == Some(index);

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
                        text(student.subject.to_string())
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
                                    text(format!("{} {}", session.day.to_string(), session.time))
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
                            text(format!(
                                "{}",
                                compute_monthly_completed_sessions(
                                    student,
                                    current_month,
                                    current_year
                                )
                            )),
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
                            text(format!(
                                "GHS {}",
                                compute_monthly_sum(
                                    student,
                                    current_month,
                                    current_year,
                                    compute_monthly_completed_sessions,
                                )
                            ))
                        ]
                        .spacing(5)
                    ),
                ]
                .spacing(10),
            ]
            .spacing(40);

            let action_section = container(
                row![
                    ui_button(
                        "Add Session",
                        12.0,
                        icons::edit(),
                        16.0,
                        18.0,
                        |_| Color::WHITE,
                        |_| Color::BLACK,
                    )
                    .padding(10)
                    .width(Length::FillPortion(2))
                    .height(Length::Fixed(40.0)),
                    ui_button(
                        "Edit",
                        12.0,
                        icons::edit(),
                        16.0,
                        18.0,
                        |theme| theme.extended_palette().background.weak.text,
                        |theme| theme.extended_palette().background.weak.color,
                    )
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
                .on_enter(Msg::StudentCardHovered(Some(index)))
                .on_exit(Msg::StudentCardHovered(None))
                .into()
        })
        .collect();

    card_list
}

fn loading_student_cards<'a>() -> Vec<Element<'a, Msg>> {
    vec![container(text!("Loading students…")).padding(20).into()]
}

fn modal<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
) -> Stack<'a, Message>
where
    Message: Clone + 'a,
{
    stack![
        base.into(),
        opaque(center(opaque(content)).style(|_theme| {
            container::Style {
                background: Some(
                    Color {
                        a: 0.8,
                        ..Color::BLACK
                    }
                    .into(),
                ),
                ..container::Style::default()
            }
        }))
    ]
    // .into()
}
