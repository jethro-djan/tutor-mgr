use std::rc::Rc;

use crate::domain::Domain;

use crate::dashboard::{self, DashboardState};
use crate::shell::{self, Screen, ShellState};
use crate::students::{self, StudentManagerState};

use iced::{Element, Subscription, Task};

pub struct App {
    pub domain: Option<Rc<Domain>>,
    pub shell: ShellState,
    pub dashboard: DashboardState,
    pub students: StudentManagerState,
}
#[derive(Clone, Debug)]
pub enum AppMsg {
    Shell(shell::Msg),
    Dashboard(dashboard::Msg),
    StudentManager(students::Msg),

    DomainLoaded(Domain),
}

impl App {
    pub fn new() -> (Self, Task<AppMsg>) {
        // let domain = Rc::new(Domain::load_state_from_db());

        let app = Self {
            domain: None,
            shell: ShellState::default(),
            dashboard: DashboardState::empty(),
            students: StudentManagerState::empty(),
        };

        let task = Task::perform(Domain::load_state_from_db(), AppMsg::DomainLoaded);

        (app, task)
    }

    pub fn update(&mut self, msg: AppMsg) -> Task<AppMsg> {
        match msg {
            AppMsg::Shell(msg) => {
                shell::update(&mut self.shell, msg);
                Task::none()
            }

            AppMsg::Dashboard(msg) => {
                dashboard::update(&mut self.dashboard, msg).map(AppMsg::Dashboard)
            }

            AppMsg::StudentManager(msg) => {
                students::update(&mut self.students, msg).map(AppMsg::StudentManager)
            }

            AppMsg::DomainLoaded(domain) => {
                let domain = Rc::new(domain);

                self.dashboard.attach_domain(&Rc::clone(&domain));
                self.students.attach_domain(Rc::clone(&domain));

                self.domain = Some(domain);

                Task::none()
            }
        }
    }

    pub fn title(&self) -> String {
        String::from("Tutor Manager")
    }

    pub fn subscription(&self) -> Subscription<AppMsg> {
        shell::subscription(&self.shell).map(AppMsg::Shell)
    }
}

impl App {
    pub fn view(&self) -> Element<'_, AppMsg> {
        let content = match self.shell.current_screen {
            Screen::Dashboard => dashboard::view(&self.dashboard).map(AppMsg::Dashboard),
            Screen::StudentManager => {
                // Placeholder until I implement students view
                students::view(&self.students).map(AppMsg::StudentManager)
            }
            Screen::Settings | Screen::Logout => {
                // Placeholder for other screens
                dashboard::view(&self.dashboard).map(AppMsg::Dashboard)
            }
        };

        shell::view(&self.shell, content, AppMsg::Shell)
    }
}
