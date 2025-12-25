use iced::{
    Element, Task, Length, Border, 
    Theme, Color, Shadow, Alignment, Vector, Font, Center,
};
use iced::mouse::Interaction;
use iced::advanced::graphics::core::font;
use iced::widget::{
    Column, Row, button, column, container, mouse_area,
    operation::focus_next, row, stack, svg, text, text_input,
};
use chrono::{Local, Datelike};
use std::rc::Rc;

use crate::ui_components::{ui_button, page_header};
use crate::icons;
use crate::domain::{
    compute_monthly_completed_sessions, compute_monthly_sum,
    get_next_session, Domain, Student,
};

pub struct StudentManagerState {
    pub search_query: String,
    pub show_add_student_modal: bool,
    pub hovered_student_card: Option<usize>,
    // pub rendered_students: Option<RenderedStudentList>,
    pub students: Vec<Student>,

    // Modal State
    // pub student_info: StudentInfo,
}

impl StudentManagerState {
    pub fn new(domain: &Rc<Domain>) -> Self {
        Self {
            search_query: String::new(),
            show_add_student_modal: false,
            hovered_student_card: None,
            // rendered_students: Some(RenderedStudentList::render_student_list(students)),
            students: domain.students.clone()

            // student_info: StudentInfo::default(),
        }
    }
}

// impl Default for StudentManagerState {
//     fn default() -> Self {
//         Self {
//             search_query: String::new(),
//             show_add_student_modal: false,
//             hovered_student_card: None,
//             student_info: StudentInfo::default(),
//         }
//     }
// }

#[derive(Clone, Debug)]
pub enum Msg {
    ShowAddStudentModal,
    CloseAddStudentModal,
    StudentCardHovered(Option<usize>),
}

// #[derive(Default)]
// pub struct StudentInfo {
//     pub first_name: String,
//     pub last_name: String,
//     pub other_names: Some(String),
// 
//     pub subject: String,
// 
//     pub pay_rate: f32,
// }

// pub struct RenderedStudentList(pub Vec<StudentInfo>);
// 
// impl RenderedStudentList {
//     pub fn render_student_list(
//         students: Vec<Student>
//     ) -> Self {
//         let rendered_students = students
//             .into_iter()
//             .map(|std| StudentInfo {
//                 first_name: std.name.first,
//                 last_name: std.name.last,
//                 other_names: Some(std.name.other),
// 
//                 subject: std.subject.to_string(),
// 
//                 pay_rate: std.payment_data.amount,
//             })
//             .collect();
// 
//         RenderedStudentList(rendered_students)
//     }
// }


pub fn update(state: &mut StudentManagerState, msg: Msg) -> Task<Msg> {
    match msg {
        Msg::ShowAddStudentModal => {
            state.show_add_student_modal = true;
            focus_next()
        },
        Msg::CloseAddStudentModal => {
            state.show_add_student_modal = false;
            Task::none()
        },
        Msg::StudentCardHovered(card_idx_opt) => {
            state.hovered_student_card = card_idx_opt;
            Task::none()
        }
    }
}

pub fn view(state: &StudentManagerState) -> Element<'_, Msg> {
    view_student_manager(state)
}


fn view_student_manager(state: &StudentManagerState) -> Element<'_, Msg> {
    let search_bar = view_search_bar("Search Students", &state.search_query);
    let add_button =
        button(svg(icons::plus()).width(25).height(25)).on_press(Msg::ShowAddStudentModal);
    let action_bar = row![search_bar, add_button].spacing(100);
    let card_container = container(
        Row::new()
            .extend(view_student_manager_card_list(state))
            .spacing(30),
    );

    let main_container = container(
        column![
            page_header("Student Manager"),
            action_bar,
            card_container,
        ]
        .spacing(30),
    )
    .width(Length::Fill)
    .height(Length::Fill);

    if state.show_add_student_modal {
        modal(main_container.into(), || {
            container(column![
                row![text!("Modal open")],
                button(text!("Close modal")).on_press(Msg::CloseAddStudentModal),
            ])
            .into()
        })
        .into()
    } else {
        main_container.into()
    }
}


fn view_search_bar<'a>(
    placeholder_text: &'a str,
    search_query_state_tracker: &'a str,
) -> Element<'a, Msg> {
    container(text_input(placeholder_text, &search_query_state_tracker)).into()
}

fn view_student_manager_card_list(state: &StudentManagerState) -> Vec<Element<'_, Msg>> {
    let today = Local::now().naive_local().date();
    let current_year = today.year();
    let current_month = today.month();

    let card_list = state
        .students
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
