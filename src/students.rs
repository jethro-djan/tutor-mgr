use chrono::{Datelike, Local, Weekday};
use iced::advanced::graphics::core::font;
use iced::mouse::Interaction;
use iced::widget::{
    Column, Row, Stack, button, center, column, container, mouse_area, opaque,
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

impl TimeSlot {
    fn new(id: usize) -> Self {
        Self {
            id,
            selected_day: None,
            selected_time: None,
        }
    }
}

pub struct StudentManagerState {
    pub search_query: String,
    pub show_add_student_modal: bool,
    pub hovered_student_card: Option<usize>,
    pub tutor: Option<Tutor>,
    pub students: Option<Vec<Student>>,
    pub modal_state: AddStudentModal,
}

impl StudentManagerState {
    pub fn attach_domain(&mut self, domain: Rc<Domain>) {
        self.search_query.clear();
        self.show_add_student_modal = false;
        self.hovered_student_card = None;
        self.tutor = Some(domain.tutor.clone());
        self.students = Some(domain.students.clone());
        self.modal_state.clear();
    }

    pub fn empty() -> Self {
        Self {
            search_query: String::new(),
            show_add_student_modal: false,
            hovered_student_card: None,
            tutor: None,
            students: None,
            modal_state: AddStudentModal::default(),
        }
    }
}

#[derive(Default)]
pub struct AddStudentModal {
    pub modal_input: ModalInput,
    pub modal_message: String,
    pub selected_subject: Option<TutorSubject>,
    pub validation_errors: Option<ValidatedStudent>,
    pub time_slots: Vec<TimeSlot>,
    pub next_slot_id: usize,
}

impl AddStudentModal {
    pub fn clear(&mut self) {
        self.modal_input = ModalInput::default();
        self.selected_subject = None;
        self.time_slots = vec![TimeSlot::new(0)];
        self.next_slot_id = 1;
        self.validation_errors = None;
        self.modal_message.clear();
    }
}

#[derive(Debug, Clone)]
pub enum StudentError {
    StudentNotSaved(ModalInput),
}

impl std::fmt::Display for StudentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StudentError::StudentNotSaved(modal_input) => {
                write!(f, "Student with name {} {} not saved", 
                    modal_input.first_name, modal_input.last_name)
            }
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
    AddStudent,
    StudentAdded(Result<(), StudentError>),
    AddTimeSlot,
    RemoveTimeSlot(usize),
    TutoringDaySelected(usize, DaySelection),
    TutoringTimeSelected(usize, TimeSelection),
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TimeSelection {
    Time(String),
}

impl std::fmt::Display for TimeSelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeSelection::Time(time) => write!(f, "{}", time),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum DaySelection {
    Day(Weekday),
}

impl std::fmt::Display for DaySelection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DaySelection::Day(day) => write!(f, "{}", day),
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct WeeklySchedule(pub Vec<SessionData>);

#[derive(Default, Debug, Clone)]
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
            state.modal_state.clear();
            state.show_add_student_modal = false;
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
                state.modal_state.time_slots.push(
                    TimeSlot::new(state.modal_state.next_slot_id)
                );
                state.modal_state.next_slot_id += 1;
            }
            Task::none()
        }
        Msg::RemoveTimeSlot(id) => {
            state.modal_state.time_slots.retain(|slot| slot.id != id);
            if state.modal_state.time_slots.is_empty() {
                state.modal_state.time_slots.push(
                    TimeSlot::new(state.modal_state.next_slot_id)
                );
                state.modal_state.next_slot_id += 1;
            }
            Task::none()
        }
        Msg::TutoringDaySelected(slot_id, day) => {
            if let Some(slot) = state.modal_state.time_slots.iter_mut().find(|s| s.id == slot_id) {
                slot.selected_day = Some(day);
                slot.selected_time = None;
            }
            Task::none()
        }
        Msg::TutoringTimeSelected(slot_id, time) => {
            if let Some(slot) = state.modal_state.time_slots.iter_mut().find(|s| s.id == slot_id) {
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
        Msg::AddStudent => {
            let validated_data = validate_student(state.modal_state.modal_input.clone());
            
            if validated_data.is_valid() {
                Task::perform(
                    add_student(state.modal_state.modal_input.clone()),
                    Msg::StudentAdded
                )
            } else {
                state.modal_state.validation_errors = Some(validated_data);
                Task::none()
            }
        }
        Msg::StudentAdded(result) => {
            state.modal_state.modal_message = match result {
                Ok(()) => "Student saved".to_string(),
                Err(e) => e.to_string(),
            };
            Task::none()
        }
    }
}

pub fn view(state: &StudentManagerState) -> Element<'_, Msg> {
    view_student_manager(state)
}

fn view_student_manager(state: &StudentManagerState) -> Element<'_, Msg> {
    let search_bar = view_search_bar("Search Students", &state.search_query);
    let add_button = create_add_student_button();
    let action_bar = row![search_bar, add_button].spacing(100);
    
    let card_container = container(
        Row::new()
            .extend(view_student_manager_card_list(state))
            .spacing(30)
    );

    let header = page_header("Student Manager");
    let main_area_content = global_content_container(
        column![action_bar, card_container].spacing(30)
    )
    .width(Length::Fill)
    .height(Length::Fill);

    let main_container = column![header, main_area_content];

    if state.show_add_student_modal {
        modal(main_container, modal_content_container(state)).into()
    } else {
        main_container.into()
    }
}

fn create_add_student_button<'a>() -> Element<'a, Msg> {
    button(
        row![
            svg(icons::plus())
                .width(22)
                .height(22)
                .style(|_theme: &Theme, _status| svg::Style {
                    color: Some(Color::from_rgba(0.0, 0.2, 0.9, 0.7)),
                }),
            text("Add Student")
                .font(Font {
                    weight: font::Weight::Medium,
                    ..Default::default()
                })
                .style(|_theme: &Theme| text::Style {
                    color: Some(Color::from_rgba(0.0, 0.2, 0.9, 0.7)),
                }),
        ]
        .align_y(Center)
        .spacing(5),
    )
    .style(|_theme, _status| button::Style {
        background: None,
        ..Default::default()
    })
    .on_press(Msg::ShowAddStudentModal)
    .into()
}

fn modal_content_container(state: &StudentManagerState) -> Element<'_, Msg> {
    let basic_info_section = create_basic_info_section(state);
    let schedule_section = create_schedule_section(state);
    let action_section = create_action_section();

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

fn create_basic_info_section(state: &StudentManagerState) -> Element<'_, Msg> {
    column![
        container(text("Basic Information").size(18).font(Font {
            weight: font::Weight::Semibold,
            ..Default::default()
        }))
        .padding([20, 0]),
        column![
            row![
                create_validated_input(
                    "First Name",
                    "John",
                    &state.modal_state.modal_input.first_name,
                    state.modal_state.validation_errors.as_ref().map(|v| &v.first),
                    Msg::FirstNameInputChanged
                ),
                create_validated_input(
                    "Last Name",
                    "Smith",
                    &state.modal_state.modal_input.last_name,
                    state.modal_state.validation_errors.as_ref().map(|v| &v.last),
                    Msg::LastNameInputChanged
                ),
                create_validated_input(
                    "Other Names",
                    "",
                    &state.modal_state.modal_input.other_names,
                    state.modal_state.validation_errors.as_ref().map(|v| &v.other),
                    Msg::OtherNamesInputChanged
                ),
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
            ]
            .padding([10, 0])
            .spacing(5),
            create_validated_input(
                "Rate per session (GHS)",
                "e.g., 150",
                &state.modal_state.modal_input.pay_rate,
                state.modal_state.validation_errors.as_ref().map(|v| &v.rate),
                Msg::RateInputChanged
            ),
        ]
        .spacing(20),
    ]
    .into()
}

fn create_validated_input<'a, F>(
    label: &'a str,
    placeholder: &'a str,
    value: &'a str,
    validation: Option<&'a (String, ValidityTag)>,
    on_input: F,
) -> Element<'a, Msg>
where
    F: Fn(String) -> Msg + 'a,
{
    let mut col = column![
        text(label).size(13).font(Font {
            weight: font::Weight::Medium,
            ..Default::default()
        }),
        text_input(placeholder, value).on_input(on_input),
    ]
    .spacing(5);

    if let Some((_, ValidityTag::Problematic { message, .. })) = validation {
        col = col.push(
            text(message)
                .size(13)
                .font(Font {
                    weight: font::Weight::Normal,
                    ..Default::default()
                })
                .style(|_theme: &Theme| text::Style {
                    color: Some(Color::from_rgb(1.0, 0.0, 0.0)),
                    ..Default::default()
                })
        );
    }

    col.into()
}

fn create_schedule_section(state: &StudentManagerState) -> Element<'_, Msg> {
    let days: Vec<DaySelection> = state
        .tutor
        .as_ref()
        .unwrap()
        .tutoring_days
        .clone()
        .into_iter()
        .map(DaySelection::Day)
        .collect();

    let mut schedule_column = column![
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
    .spacing(10);

    for slot in &state.modal_state.time_slots {
        schedule_column = schedule_column.push(create_time_slot_row(slot, days.clone(), state));
    }

    schedule_column.into()
}

fn create_time_slot_row<'a>(
    slot: &'a TimeSlot,
    days: Vec<DaySelection>,
    state: &'a StudentManagerState,
) -> Element<'a, Msg> {
    let slot_id = slot.id;
    let can_remove = state.modal_state.time_slots.len() > 1;

    let time_picker = create_time_picker(slot, state);
    let remove_button = create_remove_button(can_remove, slot_id);

    row![
        pick_list(days, slot.selected_day.clone(), move |day| {
            Msg::TutoringDaySelected(slot_id, day)
        })
        .placeholder("Select Day")
        .width(Length::FillPortion(1))
        .menu_height(155),
        space().width(Length::Fixed(20.0)),
        time_picker,
        space().width(Length::Fixed(10.0)),
        remove_button,
    ]
    .spacing(10)
    .padding([10, 0])
    .into()
}

fn create_time_picker<'a>(
    slot: &'a TimeSlot,
    state: &'a StudentManagerState,
) -> Element<'a, Msg> {
    let slot_id = slot.id;
    
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
    }
}

fn create_remove_button<'a>(can_remove: bool, slot_id: usize) -> Element<'a, Msg> {
    if can_remove {
        mouse_area(
            button(svg::Svg::new(icons::delete()).style(|_theme, _status| svg::Style {
                color: Some(Color::from_rgba(1.0, 0.0, 0.2, 1.0)),
            }))
            .padding(5)
            .width(Length::Fixed(30.0))
            .style(|theme: &Theme, _status| button::Style {
                background: Some(Background::Color(
                    theme.extended_palette().background.weak.color
                )),
                ..Default::default()
            })
            .on_press(Msg::RemoveTimeSlot(slot_id)),
        )
        .interaction(Interaction::Pointer)
        .into()
    } else {
        space().width(Length::Fixed(30.0)).into()
    }
}

fn create_action_section<'a>() -> Element<'a, Msg> {
    container(
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
                .style(|_theme, _status| button::Style {
                    border: Border {
                        color: Color::BLACK,
                        width: 1.0,
                        radius: 10.0.into(),
                    },
                    ..Default::default()
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
                .on_press(Msg::AddStudent),
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
    .align_y(Alignment::End)
    .into()
}

fn view_search_bar<'a>(placeholder: &'a str, query: &'a str) -> Element<'a, Msg> {
    container(text_input(placeholder, query)).into()
}

fn view_student_manager_card_list(state: &StudentManagerState) -> Vec<Element<'_, Msg>> {
    match &state.students {
        None => vec![container(text!("Loading students…")).padding(20).into()],
        Some(students) => render_student_cards(state, students),
    }
}

fn render_student_cards<'a>(
    state: &'a StudentManagerState,
    students: &'a [Student],
) -> Vec<Element<'a, Msg>> {
    let today = Local::now().naive_local().date();

    students
        .iter()
        .enumerate()
        .map(|(index, student)| create_student_card(state, student, index, today))
        .collect()
}

fn create_student_card<'a>(
    state: &'a StudentManagerState,
    student: &'a Student,
    index: usize,
    today: chrono::NaiveDate,
) -> Element<'a, Msg> {
    let next_session = get_next_session(student);
    let is_hovered = state.hovered_student_card == Some(index);

    let title_section = create_card_title(student);
    let main_section = create_card_main_section(student, next_session, today);
    let action_section = create_card_actions();

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
}

fn create_card_title<'a>(student: &'a Student) -> Element<'a, Msg> {
    let full_name = if let Some(other) = &student.name.other {
        format!("{} {} {}", student.name.first, other, student.name.last)
    } else {
        format!("{} {}", student.name.first, student.name.last)
    };

    row![column![
        text(full_name)
            .font(Font {
                weight: font::Weight::Bold,
                ..Default::default()
            })
            .size(20),
        text(student.subject.to_string())
            .font(Font {
                weight: font::Weight::Light,
                ..Default::default()
            })
            .size(15),
    ]
    .align_x(Alignment::Start)
    .width(Length::Fill)
    .spacing(5)]
    .height(Length::Fixed(50.0))
    .into()
}

fn create_card_main_section<'a>(
    student: &'a Student,
    next_session: chrono::NaiveDate,
    today: chrono::NaiveDate,
) -> Element<'a, Msg> {
    let day = next_session.format("%A").to_string();
    let date = next_session.format("%d %B %Y").to_string();

    column![
        create_info_row(
            icons::calendar(),
            "Schedule",
            Column::new()
                .extend(student.tabled_sessions.iter().map(|session| {
                    text(format!("{} {}", session.day, session.time)).into()
                }))
                .spacing(2)
        ),
        create_info_row(
            icons::schedule(),
            "Next session",
            column![text(format!("{}, {}", day, date))].spacing(5)
        ),
        create_info_row(
            icons::check_circle(),
            "Completed sessions",
            column![text(format!(
                "{}",
                compute_monthly_completed_sessions(student, today.month(), today.year())
            ))]
            .spacing(5)
        ),
        create_info_row(
            icons::payments(),
            "Amount accrued",
            column![text(format!(
                "GHS {}",
                compute_monthly_sum(
                    student,
                    today.month(),
                    today.year(),
                    compute_monthly_completed_sessions,
                )
            ))]
            .spacing(5)
        ),
    ]
    .spacing(40)
    .into()
}

fn create_info_row<'a, C>(icon: svg::Handle, label: &'a str, content: C) -> Element<'a, Msg>
where
    C: Into<Element<'a, Msg>>,
{
    row![
        container(svg::Svg::new(icon).width(22).height(22))
            .align_y(Alignment::Center)
            .height(Length::Fixed(30.0)),
        container(column![
            text(label)
                .font(Font {
                    weight: font::Weight::Normal,
                    ..Default::default()
                })
                .size(12),
            content.into(),
        ]
        .spacing(4)),
    ]
    .spacing(10)
    .into()
}

fn create_card_actions<'a>() -> Element<'a, Msg> {
    container(
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
    .align_y(Alignment::Start)
    .into()
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
        opaque(center(opaque(content)).style(|_theme| container::Style {
            background: Some(Color { a: 0.8, ..Color::BLACK }.into()),
            ..Default::default()
        }))
    ]
}

#[derive(PartialEq, Debug)]
pub enum ValidityTag {
    Safe,
    Problematic {
        error_type: ValidityError,
        message: String,
    },
}

#[derive(PartialEq, Debug)]
pub enum ValidityError {
    Empty,
    NotANumber,
    TooLong,
    TooShort,
    ContainsNonLetters,
}

pub struct ValidatedStudent {
    first: (String, ValidityTag),
    last: (String, ValidityTag),
    other: (String, ValidityTag),
    rate: (String, ValidityTag),
}

impl ValidatedStudent {
    fn is_valid(&self) -> bool {
        matches!(self.first.1, ValidityTag::Safe)
            && matches!(self.last.1, ValidityTag::Safe)
            && matches!(self.other.1, ValidityTag::Safe)
            && matches!(self.rate.1, ValidityTag::Safe)
    }
}

fn validate_student(modal_input: ModalInput) -> ValidatedStudent {
    ValidatedStudent {
        first: validate_name(modal_input.first_name),
        last: validate_name(modal_input.last_name),
        other: validate_optional_field(modal_input.other_names, 100),
        rate: validate_number(modal_input.pay_rate),
    }
}

fn validate_name(name: String) -> (String, ValidityTag) {
    let (name, tag) = validate_length(name, 2, 50);
    if !matches!(tag, ValidityTag::Safe) {
        return (name, tag);
    }
    validate_letters_only(name)
}

fn validate_length(input: String, min: usize, max: usize) -> (String, ValidityTag) {
    let (input, tag) = validate_empty(input);
    if !matches!(tag, ValidityTag::Safe) {
        return (input, tag);
    }

    if input.len() < min {
        return (
            input,
            ValidityTag::Problematic {
                error_type: ValidityError::TooShort,
                message: format!("Must be at least {} characters", min),
            },
        );
    }

    if input.len() > max {
        return (
            input,
            ValidityTag::Problematic {
                error_type: ValidityError::TooLong,
                message: format!("Must be no more than {} characters", max),
            },
        );
    }

    (input, ValidityTag::Safe)
}

fn validate_empty(input: String) -> (String, ValidityTag) {
    let input = input.trim().to_string();
    if input.is_empty() {
        return (
            input,
            ValidityTag::Problematic {
                error_type: ValidityError::Empty,
                message: "Field cannot be empty".to_string(),
            },
        );
    }
    (input, ValidityTag::Safe)
}

fn validate_letters_only(input: String) -> (String, ValidityTag) {
    if !input.chars().all(|c| c.is_alphabetic() || c.is_whitespace()) {
        return (
            input,
            ValidityTag::Problematic {
                error_type: ValidityError::ContainsNonLetters,
                message: "Name should only contain letters".to_string(),
            },
        );
    }
    (input, ValidityTag::Safe)
}

fn validate_number(rate: String) -> (String, ValidityTag) {
    let trimmed = rate.trim().to_string();
    if trimmed.is_empty() {
        return (
            trimmed,
            ValidityTag::Problematic {
                error_type: ValidityError::Empty,
                message: "Rate cannot be empty".to_string(),
            },
        );
    }

    match trimmed.parse::<f32>() {
        Ok(_) => (trimmed, ValidityTag::Safe),
        Err(_) => (
            trimmed,
            ValidityTag::Problematic {
                error_type: ValidityError::NotANumber,
                message: "Must be a valid number".to_string(),
            },
        ),
    }
}

fn validate_optional_field(input: String, max: usize) -> (String, ValidityTag) {
    let input = input.trim().to_string();

    if input.is_empty() {
        return (input, ValidityTag::Safe);
    }

    if input.len() > max {
        return (
            input,
            ValidityTag::Problematic {
                error_type: ValidityError::TooLong,
                message: format!("Must be no more than {} characters", max),
            },
        );
    }

    (input, ValidityTag::Safe)
}

async fn add_student(_modal_input: ModalInput) -> Result<(), StudentError> {
    Ok(())
}
