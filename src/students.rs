use iced::{widget::operation::focus_next, Element, Task};

pub struct StudentManagerState {
    pub search_query: String,
    pub show_add_student_modal: bool,
    pub hovered_student_card: Option<usize>,

    // Modal State
    pub student_info: StudentInfo,
}

impl Default for StudentManagerState {
    fn default() -> Self {
        Self {
            search_query: String::new(),
            show_add_student_modal: false,
            hovered_student_card: None,
            student_info: StudentInfo::default(),
        }
    }
}

pub enum Msg {
    ShowAddStudentModal,
    CloseAddStudentModal,
    StudentCardHovered(Option<usize>),
}

#[derive(Default)]
pub struct StudentInfo {
    pub first_name: String,
    pub last_name: String,
    pub other_names: String,

    pub subject: String,

    pub rate_per_session: String,
}

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
