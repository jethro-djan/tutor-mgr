use std::rc::Rc;

use crate::domain::Domain;


use crate::shell::{self, ShellState, Screen};
use crate::dashboard::{self, DashboardState};

use iced::{Element, Task, Subscription};

pub struct App {
    pub domain: Option<Rc<Domain>>,
    pub shell: ShellState,
    pub dashboard: DashboardState,
    // pub students: StudentState,
}

pub enum AppMsg {
    Shell(shell::Msg),
    Dashboard(dashboard::Msg),
    // Students(students::Msg),
}

impl App {
    pub fn new() -> (Self, Task<AppMsg>) {
        let domain = Rc::new(Domain::load_state_from_db());

        let app = Self {
            domain: None,
            shell: ShellState::default(),
            dashboard: DashboardState::new(&domain),
        };

        (app, Task::none())
    }

    pub fn update(&mut self, msg: AppMsg) -> Task<AppMsg> {
        match msg {
            AppMsg::Shell(msg) => {
                shell::update(&mut self.shell, msg);
                Task::none()
            }

            AppMsg::Dashboard(msg) => {
                dashboard::update(&mut self.dashboard, msg)
                    .map(AppMsg::Dashboard)
            }

            // AppMsg::Students(msg) => {
            //     students::update(&mut self.students, msg, &self.domain);
            //     Task::none()
            // }
        }
    }

    pub fn title(&self) -> String {
        String::from("Tutor Manager")
    }

    pub fn subscription(&self) -> Subscription<AppMsg> {
        shell::subscription(&self.shell)
            .map(AppMsg::Shell)
    }
}

impl App {
    pub fn view(&self) -> Element<'_, AppMsg> {
        let content = match self.shell.current_screen {
            Screen::Dashboard => {
                dashboard::view(&self.dashboard)
                    .map(AppMsg::Dashboard)
            }
            Screen::StudentManager => {
                // Placeholder until I implement students view
                dashboard::view(&self.dashboard)
                    .map(AppMsg::Dashboard)
            }
            Screen::Settings | Screen::Logout => {
                // Placeholder for other screens
                dashboard::view(&self.dashboard)
                    .map(AppMsg::Dashboard)
            }
        };

        shell::view(&self.shell, content, AppMsg::Shell)
    }
}


