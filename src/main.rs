use chrono::{DateTime, Datelike, Duration, Local, Month, NaiveDate, TimeZone, Weekday, };
use iced::advanced::graphics::core::font;
use iced::mouse::Interaction;
use iced::widget::canvas::{self, Frame, Path, Stroke, Text};
use iced::widget::{
    Canvas, Column, Container, Row, button, column, container, operation::focus_next, mouse_area, row, stack,
    svg, text, text_input,
};
use iced::window::frames;
use iced::{
    Alignment, Background, Border, Center, Color, Element, Font, Length, Point, Rectangle,
    Renderer, Shadow, Size, Subscription, Task, Theme, Vector,
};
use lilt::{Animated, Easing};
use std::time::Instant;

// =========================================
// DOMAIN MODELS AND LOGIC
// =========================================
struct Domain {
    tutor: Tutor,
    students: Vec<Student>,
    monthly_summaries: Vec<MonthlySummary>,
}

impl Domain {
    pub fn compute_trend_history(&self) -> Vec<TrendData> {
        compute_trend_history_internal(&self.monthly_summaries)
    }
}

#[derive(Copy, Clone)]
struct MonthlySummary {
    year_month: YearMonth,
    actual_revenue: f32,
    potential_revenue: f32,
    total_scheduled_sessions: usize,
    total_actual_sessions: usize,
}

#[derive(Clone)]
struct TrendData {
    revenue_trend: ActualRevenueTrendData,
    sessions_trend: ActualSessionTrendData,
}

#[derive(Clone)]
struct ActualRevenueTrendData {
    trend: NumberTrend,
    current_revenue: f32,
    previous_revenue: f32,
    year_month: YearMonth,
}

#[derive(Clone)]
struct ActualSessionTrendData {
    trend: NumberTrend,
    current_sessions: f32,
    previous_sessions: f32,
    year_month: YearMonth,
}

type TrendHistory = Vec<TrendData>;

#[derive(Copy, Clone)]
struct YearMonth {
    year: i32,
    month: Month,
}

impl Domain {
    fn load_from_db() -> Self {
        mock_domain()
    }
}

struct Student {
    id: String,
    name: PersonalName,
    subject: TutorSubject,
    tabled_sessions: Vec<SessionData>,
    actual_sessions: Vec<DateTime<Local>>,

    payment_data: PaymentData,
}

struct Tutor {
    id: String,
    name: PersonalName,
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
    payment_type: PaymentType,
    amount: f32,
}

enum PaymentType {
    PerSession,
    Monthly,
}

fn compute_accrued_amount(student: &Student) -> f32 {
    match student.payment_data.payment_type {
        PaymentType::PerSession => {
            let no_of_days = compute_num_of_completed_sessions(student);
            student.payment_data.amount * (no_of_days as f32)
        }
        PaymentType::Monthly => student.payment_data.amount
    }
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

/// Computes month-over-month trends for some eligible data
fn compute_trend_history_internal(monthly_summaries: &[MonthlySummary]) -> TrendHistory {
    if monthly_summaries.len() < 2 {
        return Vec::<TrendData>::new()
    }

    let mut sorted_summaries = monthly_summaries.to_vec();
    sorted_summaries.sort_by_key(|summary| summary.year_month.month);

    let mut revenue_trend = Vec::new();
    let mut sessions_trend = Vec::new();
    
    let mut trend_history = Vec::new();

    for i in 1..monthly_summaries.len() {
        let previous = monthly_summaries[i -1];
        let current = monthly_summaries[i];

        revenue_trend.push(ActualRevenueTrendData {
            trend: compute_trend(previous.actual_revenue, current.actual_revenue),
            current_revenue: current.actual_revenue,
            previous_revenue: previous.actual_revenue,
            year_month: current.year_month,
        });

        sessions_trend.push(ActualSessionTrendData {
            trend: compute_trend(previous.actual_revenue, current.actual_revenue),
            current_sessions: current.actual_revenue,
            previous_sessions: previous.actual_revenue,
            year_month: current.year_month,
        });

        trend_history.push(TrendData {
            revenue_trend: ActualRevenueTrendData {
                trend: compute_trend(previous.actual_revenue, current.actual_revenue),
                current_revenue: current.actual_revenue,
                previous_revenue: previous.actual_revenue,
                year_month: current.year_month,
            },
            sessions_trend: ActualSessionTrendData { 
                trend: compute_trend(previous.total_actual_sessions as f32, current.total_scheduled_sessions as f32),
                current_sessions: current.total_actual_sessions as f32, 
                previous_sessions: previous.total_actual_sessions as f32, 
                year_month: current.year_month, 
            }
        })
    }

    trend_history
}

fn compute_trend(previous: f32, current: f32) -> NumberTrend {
    let percentage_change = if previous == 0.0 {
        0.0 
    } else {
        (current - previous) / previous * 100.0
    };

    NumberTrend { 
        trend_direction: if percentage_change >= 0.0 {
            TrendDirection::Up
        } else {
            TrendDirection::Down
        },
        percentage_change: percentage_change.abs(),
    }
}


// =========================================
// APPLICATION STATE
// =========================================
struct TutoringManager {
    domain: Domain,
    current_screen: Screen,
    ui: UIState,
}

#[derive(Default)]
struct StudentInfo {
    first_name: String,
    last_name: String,
    other_names: String,

    subject: String,

    rate_per_session: String,
}

struct UIState {
    // Navigation
    selected_menu_item: SideMenuItem,
    hovered_menu_item: Option<SideMenuItem>,
    side_menu_hovered: bool,

    // Side Menu Layout
    animated_menu_width_change: Animated<bool, Instant>,
    animated_menu_item_height_change: Animated<bool, Instant>,
    show_menu_text: bool,

    // Dashboard State
    hovered_dashboard_card: Option<usize>,
    barchart: GroupedBarChart,
    linechart: LineChart,
    dashboard_summary: DashboardSummary,

    // StudentManager State
    search_query: String,
    show_add_student_modal: bool,
    hovered_student_card: Option<usize>,

    // Modal State
    student_info: StudentInfo,
}

/// Info about metric trend from previous month
///
/// Answers the question of whether some metric has increased or decreased from previous
/// amount
#[derive(Clone)]
struct NumberTrend {
    trend_direction: TrendDirection,
    percentage_change: f32,
}

#[derive(Clone)]
enum TrendDirection {
    Up,
    Down,
}

struct ActualRevenueSummary {
    amount: f32,
    trend: NumberTrend,
}

struct PotentialRevenueSummary {
    amount: f32,
}

struct LostRevenueSummary {
    amount: f32,
    trend: NumberTrend,
}

struct AttendanceSummary {
    total_scheduled_sessions: usize,
    total_actual_sessions: usize,
}

struct MonthlySummaryWithTrend {
    month: YearMonth,

    attendance: AttendanceSummary,
    actual_revenue: ActualRevenueSummary,
    potential_revenue: PotentialRevenueSummary,
    lost_revenue: LostRevenueSummary,
}


struct DashboardSummary {
    attendance: AttendanceSummary,
    actual_revenue: ActualRevenueSummary,
    potential_revenue: PotentialRevenueSummary,
    lost_revenue: LostRevenueSummary,
}


impl DashboardSummary {
    fn compute_from_domain_state() -> Self {
        let domain = Domain::load_from_db();
        let total_actual_sessions = domain.students
            .iter()
            .map(|student| compute_num_of_completed_sessions(student) as usize)
            .sum();
        let total_scheduled_sessions = domain.students
            .iter()
            .map(|student| student.actual_sessions.len())
            .sum();
        let potential_earnings = domain.students
            .iter()
            .map(|student| {
                let amount = student.payment_data.amount;
                match student.payment_data.payment_type {
                    PaymentType::Monthly => amount,
                    PaymentType::PerSession => (total_scheduled_sessions as f32) * amount,
                }
            })
            .sum();
        let actual_earnings = domain.students
            .iter()
            .map(compute_accrued_amount)
            .sum();

        let attendance = AttendanceSummary {
            total_actual_sessions,
            total_scheduled_sessions,
        };
        let actual_revenue = ActualRevenueSummary {
            amount: actual_earnings,
            trend: NumberTrend { trend_direction: TrendDirection::Up, percentage_change: 5.0 }, // placeholder
        };
        let potential_revenue = PotentialRevenueSummary {
            amount: potential_earnings,
        };
        let lost_revenue = LostRevenueSummary {
            amount: potential_earnings - actual_earnings,
            trend: NumberTrend { trend_direction: TrendDirection::Down, percentage_change: 5.0 } // placeholder
        };

        Self {
            attendance,
            actual_revenue,
            potential_revenue,
            lost_revenue,
        }
    }
}


#[derive(Debug)]
enum Screen {
    Dashboard,
    StudentManager,
    Settings,
    Logout,
}

#[derive(Debug, Clone, PartialEq, Copy)]
enum SideMenuItem {
    Dashboard,
    StudentManager,
    Settings,
    Logout,
}

#[derive(Debug, Clone)]
enum Message {
    // Navigation
    NavigateToScreen(SideMenuItem),
    MenuItemHovered(Option<SideMenuItem>),
    SideMenuHovered(bool),

    // Dashboard
    DashboardCardHovered(Option<usize>),
    Tick,

    // Student Manager
    ShowAddStudentModal,
    CloseAddStudentModal,
    StudentCardHovered(Option<usize>),
}

// =========================================
// APPLICATION LOGIC
// =========================================

impl TutoringManager {
    fn new() -> (Self, Task<Message>) {
        let income_data = mock_income_data();
        let attendance_data = mock_attendance_data();

        (
            Self {
                domain: Domain::load_from_db(),
                current_screen: Screen::Dashboard,
                ui: UIState {
                    selected_menu_item: SideMenuItem::Dashboard,
                    hovered_menu_item: None,
                    side_menu_hovered: false,

                    animated_menu_width_change: Animated::new(false)
                        .duration(300.)
                        .easing(Easing::EaseInOut),
                    animated_menu_item_height_change: Animated::new(false)
                        .duration(200.)
                        .easing(Easing::EaseInOut),
                    show_menu_text: false,

                    hovered_dashboard_card: None,
                    barchart: GroupedBarChart::default(income_data),
                    linechart: LineChart::default(attendance_data),
                    dashboard_summary: DashboardSummary::compute_from_domain_state(),

                    search_query: String::new(),
                    show_add_student_modal: false,
                    hovered_student_card: None,

                    student_info: StudentInfo::default(),
                },
            },
            Task::none(),
        )
    }

    fn title(&self) -> String {
        String::from("Tutor Manager")
    }

    fn subscription(&self) -> Subscription<Message> {
        let now = Instant::now();
        if self.ui.animated_menu_width_change.in_progress(now) {
            frames().map(|_| Message::Tick)
        } else {
            Subscription::none()
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Navigation Messages
            Message::NavigateToScreen(menu_item) => self.handle_navigation(menu_item),
            Message::MenuItemHovered(menu_item_opt) => self.handle_menu_item_hover(menu_item_opt),
            Message::SideMenuHovered(is_hovered) => self.handle_side_menu_hover(is_hovered),

            // Dashboard Messages
            Message::DashboardCardHovered(card_index) => {
                self.ui.hovered_dashboard_card = card_index;
                Task::none()
            }
            Message::Tick => Task::none(),

            // StudentManager Messages
            Message::ShowAddStudentModal => self.handle_show_add_student_modal(),
            Message::CloseAddStudentModal => self.handle_close_add_student_modal(),
            Message::StudentCardHovered(card_index) => self.handle_student_card_hover(card_index),
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let main_content = container(self.view_current_screen()).padding([50, 20]);

        let layout = row![self.view_side_menu(), main_content,].spacing(20);

        container(layout).into()
    }

    fn view_current_screen(&self) -> Element<'_, Message> {
        match self.current_screen {
            Screen::Dashboard => self.view_dashboard(),
            Screen::StudentManager => self.view_student_manager(),
            Screen::Settings => self.view_settings(),
            Screen::Logout => self.view_logout(),
        }
    }
}

// Update helpers
impl TutoringManager {
    fn handle_navigation(&mut self, menu_item: SideMenuItem) -> Task<Message> {
        self.ui.selected_menu_item = menu_item.clone();
        self.current_screen = match menu_item {
            SideMenuItem::Dashboard => Screen::Dashboard,
            SideMenuItem::StudentManager => Screen::StudentManager,
            SideMenuItem::Settings => Screen::Settings,
            SideMenuItem::Logout => Screen::Logout,
        };
        Task::none()
    }

    fn handle_menu_item_hover(&mut self, menu_item_opt: Option<SideMenuItem>) -> Task<Message> {
        self.ui.hovered_menu_item = menu_item_opt.clone();
        Task::none()
    }

    fn handle_side_menu_hover(&mut self, is_hovered: bool) -> Task<Message> {
        let now = Instant::now();

        self.ui
            .animated_menu_width_change
            .transition(is_hovered, now);
        if is_hovered {
            self.ui.side_menu_hovered = true;
            self.ui.show_menu_text = true;
        } else {
            self.ui.side_menu_hovered = false;
            self.ui.show_menu_text = false;
        }

        Task::none()
    }

    fn handle_show_add_student_modal(&mut self) -> Task<Message> {
        self.ui.show_add_student_modal = true;
        focus_next()
    }

    fn handle_close_add_student_modal(&mut self) -> Task<Message> {
        self.ui.show_add_student_modal = false;
        Task::none()
    }

    fn handle_student_card_hover(&mut self, card_index: Option<usize>) -> Task<Message> {
        self.ui.hovered_student_card = card_index;
        Task::none()
    }
}

// Top-level Views
impl TutoringManager {
    fn view_dashboard(&self) -> Element<'_, Message> {
        struct CardInfo {
            title: String,
            value: String,
            trend: Option<(String, bool)>,
            hovered_dashboard: Option<usize>,
            variant: DashboardCardVariant,
        }

        let summary = &self.ui.dashboard_summary;

        let attendance_rate = if summary.attendance.total_scheduled_sessions > 0 {
            format!("{:.0}%", 
                summary.attendance.total_actual_sessions as f32 / summary.attendance.total_scheduled_sessions as f32 * 100.0
            )
        } else {
            "--".to_string()
        };

        let trend_format = |trend: &NumberTrend| -> (String, bool) {
            match trend.trend_direction {
                TrendDirection::Up => (format!("{:.1}", trend.percentage_change), true),
                TrendDirection::Down => (format!("{:.1}", trend.percentage_change), false),
            }
        };

        let card_data = [
            CardInfo {
                title: "Attendance Rate".into(),
                value: attendance_rate,
                trend: Some(trend_format(&summary.actual_revenue.trend)),
                hovered_dashboard: self.ui.hovered_dashboard_card,
                variant: DashboardCardVariant::Attendance,
            },
            CardInfo {
                title: "Actual Earnings".into(),
                value: format!("GHS {:.2}", summary.actual_revenue.amount),
                trend: Some(trend_format(&summary.actual_revenue.trend)),
                hovered_dashboard: self.ui.hovered_dashboard_card,
                variant: DashboardCardVariant::ActualEarnings,
            },
            CardInfo {
                title: "Potential Earnings".into(),
                value: format!("GHS {:.2}", summary.potential_revenue.amount),
                trend: None,
                hovered_dashboard: self.ui.hovered_dashboard_card,
                variant: DashboardCardVariant::PotentialEarnings,
            },
            CardInfo {
                title: "Revenue Lost".into(),
                value: format!("GHS {:.2}", summary.lost_revenue.amount),
                trend: Some(trend_format(&summary.lost_revenue.trend)),
                hovered_dashboard: self.ui.hovered_dashboard_card,
                variant: DashboardCardVariant::RevenueLost,
            },
        ];

        let summary_section_title = text("Summary").size(14).font(Font {
            weight: font::Weight::Medium,
            ..Default::default()
        });

        let summary_cards_row = row(card_data.iter().enumerate().map(|(index, card)| {
            let is_hovered = card.hovered_dashboard == Some(index);
            metric_card(
                card.title.clone(),
                card.value.to_owned(),
                card.trend.clone(),
                is_hovered,
                Some(index),
                card.variant,
            )
        }))
        .spacing(16);

        let summary_section = column![
            summary_section_title,
            container(summary_cards_row)
                .align_x(Center)
                .max_width(900),
        ]
        .spacing(12);

        let attendance_trend_chart = self.view_trend_chart();
        let potential_vs_actual_chart = self.view_grouped_chart();

        let graphs_section_title = text("Analytics").size(14).font(Font {
            weight: font::Weight::Medium,
            ..Default::default()
        });
        let graphs = row![attendance_trend_chart, potential_vs_actual_chart,].spacing(16);

        let graph_section = column![graphs_section_title, graphs,].spacing(12);

        container(
            Column::new()
                .spacing(40)
                .padding(20)
                .push(self.view_page_header("Dashboard"))
                .push(summary_section)
                .push(graph_section),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
    }

    fn view_student_manager(&self) -> Element<'_, Message> {
        let search_bar = self.view_search_bar("Search Students", &self.ui.search_query);
        let add_button =
            button(svg(icons::plus()).width(25).height(25)).on_press(Message::ShowAddStudentModal);
        let action_bar = row![search_bar, add_button].spacing(100);
        let card_container = container(
            Row::new()
                .extend(self.view_student_manager_card_list())
                .spacing(30),
        );

        let main_container = container(
            column![
                self.view_page_header("Student Manager"),
                action_bar,
                card_container,
            ]
            .spacing(30),
        )
        .width(Length::Fill)
        .height(Length::Fill);

        if self.ui.show_add_student_modal {
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
    }

    fn view_settings(&self) -> Element<'_, Message> {
        column![self.view_page_header("Settings"),].into()
    }

    fn view_logout(&self) -> Element<'_, Message> {
        column![self.view_page_header("Logout"),].into()
    }
}

// Component views
impl TutoringManager {
    fn view_page_header(&self, header_text: &str) -> Element<'_, Message> {
        let page_title_text = text(format!("{}", header_text))
            .size(20)
            .font(Font {
                weight: font::Weight::Bold,
                ..Default::default()
            })
            .size(24);
        let page_title = row![page_title_text];
        row![page_title].into()
    }

    fn view_side_menu(&self) -> Element<'_, Message> {
        let is_selected = |item: SideMenuItem| self.ui.selected_menu_item == item;
        let is_hovered = |item: SideMenuItem| self.ui.hovered_menu_item == Some(item);

        let dash_icon = svg::Svg::new(icons::dashboard().clone())
            .width(25)
            .height(25)
            .style(move |_theme: &Theme, _status: svg::Status| {
                styles::menu_icon_style(is_hovered(SideMenuItem::Dashboard))
            });

        let student_icon = svg::Svg::new(icons::student_manager().clone())
            .width(25)
            .height(25)
            .style(move |_theme: &Theme, _status: svg::Status| {
                styles::menu_icon_style(is_hovered(SideMenuItem::StudentManager))
            });

        let settings_icon = svg::Svg::new(icons::settings().clone())
            .width(25)
            .height(25)
            .style(move |_theme: &Theme, _status: svg::Status| {
                styles::menu_icon_style(is_hovered(SideMenuItem::Settings))
            });

        let logout_icon = svg::Svg::new(icons::logout().clone())
            .width(25)
            .height(25)
            .style(move |_theme: &Theme, _status: svg::Status| {
                styles::menu_icon_style(is_hovered(SideMenuItem::Logout))
            });

        let now = Instant::now();

        mouse_area(
            container(
                column![
                    self.view_logo(),
                    column![
                        mouse_area(menu_item_container(
                            dash_icon,
                            "Dashboard",
                            is_selected(SideMenuItem::Dashboard),
                            is_hovered(SideMenuItem::Dashboard),
                            self.ui.side_menu_hovered,
                            &self.ui.animated_menu_item_height_change,
                            now,
                        ))
                        .interaction(Interaction::Pointer)
                        .on_press(Message::NavigateToScreen(SideMenuItem::Dashboard))
                        .on_enter(Message::MenuItemHovered(Some(SideMenuItem::Dashboard)))
                        .on_exit(Message::MenuItemHovered(None)),
                        mouse_area(menu_item_container(
                            student_icon,
                            "Student Manager",
                            is_selected(SideMenuItem::StudentManager),
                            is_hovered(SideMenuItem::StudentManager),
                            self.ui.side_menu_hovered,
                            &self.ui.animated_menu_item_height_change,
                            now,
                        ))
                        .interaction(Interaction::Pointer)
                        .on_press(Message::NavigateToScreen(SideMenuItem::StudentManager))
                        .on_enter(Message::MenuItemHovered(Some(SideMenuItem::StudentManager)))
                        .on_exit(Message::MenuItemHovered(None)),
                    ]
                    .spacing(5),
                    container(
                        column![
                            mouse_area(menu_item_container(
                                settings_icon,
                                "Settings",
                                is_selected(SideMenuItem::Settings),
                                is_hovered(SideMenuItem::Settings),
                                self.ui.side_menu_hovered,
                                &self.ui.animated_menu_item_height_change,
                                now,
                            ))
                            .interaction(Interaction::Pointer)
                            .on_press(Message::NavigateToScreen(SideMenuItem::Settings))
                            .on_enter(Message::MenuItemHovered(Some(SideMenuItem::Settings)))
                            .on_exit(Message::MenuItemHovered(None)),
                            mouse_area(menu_item_container(
                                logout_icon,
                                "Logout",
                                is_selected(SideMenuItem::Logout),
                                is_hovered(SideMenuItem::Logout),
                                self.ui.side_menu_hovered,
                                &self.ui.animated_menu_item_height_change,
                                now,
                            ))
                            .interaction(Interaction::Pointer)
                            .on_press(Message::NavigateToScreen(SideMenuItem::Logout))
                            .on_enter(Message::MenuItemHovered(Some(SideMenuItem::Logout)))
                            .on_exit(Message::MenuItemHovered(None)),
                        ]
                        .spacing(5)
                    )
                    .align_bottom(Length::Fill)
                ]
                .spacing(20),
            )
            .padding([20, 0])
            .width(
                self.ui
                    .animated_menu_width_change
                    .animate_bool(70.0, 180.0, now),
            )
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
            }),
        )
        .interaction(Interaction::Pointer)
        .on_enter(Message::SideMenuHovered(true))
        .on_exit(Message::SideMenuHovered(false))
        .into()
    }

    fn view_logo(&self) -> Element<'_, Message> {
        let logo_handle = if self.ui.side_menu_hovered {
            icons::logo_expanded()
        } else {
            icons::logo()
        };

        let logo = svg(logo_handle)
            .width(if self.ui.side_menu_hovered {
                140
            } else {
                40
            })
            .height(40);

        container(logo)
            .center_x(Length::Fill)
            .padding([10, 0])
            .into()
    }

    fn view_search_bar(
        &self,
        placeholder_text: &str,
        search_query_state_tracker: &str,
    ) -> Element<'_, Message> {
        container(text_input(placeholder_text, &search_query_state_tracker)).into()
    }

    fn view_student_manager_card_list(&self) -> Vec<Element<'_, Message>> {
        let card_list = self
            .domain
            .students
            .iter()
            .enumerate()
            .map(|(index, student)| {
                let next_session = get_next_session(student);
                let day = next_session.format("%A").to_string();
                let date = next_session.format("%d %B %Y").to_string();

                let is_hovered = self.ui.hovered_student_card == Some(index);

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
                                text(format!("{}", compute_num_of_completed_sessions(student))),
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
                                text(format!("GHS {}", compute_accrued_amount(student)))
                            ]
                            .spacing(5)
                        ),
                    ]
                    .spacing(10),
                ]
                .spacing(40);

                let action_section = container(
                    row![
                        button(
                            container(
                                row![
                                    svg::Svg::new(icons::plus().clone())
                                        .width(16)
                                        .height(18)
                                        .style(|theme: &Theme, _status: svg::Status| {
                                            let palette = theme.extended_palette();
                                            svg::Style {
                                                color: Some(Color::WHITE),
                                                // color: Some(palette.background.weak.text),
                                            }
                                        }),
                                    text("Add Session")
                                        .size(12)
                                        .font(Font {
                                            weight: font::Weight::Semibold,
                                            ..Default::default()
                                        })
                                        .style(|theme: &Theme| {
                                            let palette = theme.extended_palette();
                                            text::Style {
                                                color: Some(Color::WHITE),
                                                // color: Some(palette.background.weak.text),
                                            }
                                        }),
                                ]
                                .spacing(5)
                                .align_y(Center)
                            )
                            .align_x(Center)
                        )
                        .style(|theme: &Theme, _status: button::Status| {
                            let palette = theme.extended_palette();
                            button::Style {
                                background: Some(Background::Color(Color::BLACK)),
                                // background: Some(Background::Color(palette.primary.base.color)),
                                border: Border {
                                    radius: 10.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        })
                        .padding(10)
                        .width(Length::FillPortion(2))
                        .height(Length::Fixed(40.0)),
                        button(
                            container(
                                row![
                                    svg::Svg::new(icons::edit().clone())
                                        .width(16)
                                        .height(18)
                                        .width(16)
                                        .height(18)
                                        .style(|theme: &Theme, _status: svg::Status| {
                                            let palette = theme.extended_palette();
                                            svg::Style {
                                                color: Some(palette.background.weak.text),
                                            }
                                        }),
                                    text("Edit").size(12).font(Font {
                                        weight: font::Weight::Semibold,
                                        ..Default::default()
                                    })
                                ]
                                .spacing(5)
                                .align_y(Center)
                            )
                            .align_x(Center)
                        )
                        .style(|theme: &Theme, _status: button::Status| {
                            let palette = theme.extended_palette();
                            button::Style {
                                background: Some(Background::Color(palette.background.weak.color)),
                                border: Border {
                                    radius: 10.0.into(),
                                    ..Default::default()
                                },
                                ..Default::default()
                            }
                        })
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
                    .on_enter(Message::StudentCardHovered(Some(index)))
                    .on_exit(Message::StudentCardHovered(None))
                    .into()
            })
            .collect();

        card_list
    }

    fn view_grouped_chart(&self) -> Element<'_, Message> {
        let chart = Canvas::new(&self.ui.barchart)
            .width(Length::Fill)
            .height(Length::Fill);

        container(column![
            text!("Current Month: Actual vs Potential Earnings").size(20),
            chart
        ])
        .width(Length::FillPortion(3))
        .height(Length::Fixed(400.0))
        .padding(20)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();

            container::Style {
                background: Some(palette.background.weak.color.into()),
                // border: (),
                ..Default::default()
            }
        })
        .into()
    }

    fn view_trend_chart(&self) -> Element<'_, Message> {
        let chart = Canvas::new(&self.ui.linechart)
            .width(Length::Fill)
            .height(Length::Fill);

        container(column![
            text!("Attendance Rate: Last 3 Months").size(20),
            chart
        ])
        .width(Length::FillPortion(2))
        .height(Length::Fixed(400.0))
        .padding(20)
        .style(|theme: &Theme| {
            let palette = theme.extended_palette();

            container::Style {
                background: Some(palette.background.weak.color.into()),
                // border: (),
                ..Default::default()
            }
        })
        .into()
    }
}

// =========================================
// REUSABLE UI components
// =========================================
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

#[derive(Clone, Copy)]
enum DashboardCardVariant {
    Attendance,
    ActualEarnings,
    PotentialEarnings,
    RevenueLost,
}

fn metric_card<'a>(
    title: String,
    value: String,
    trend: Option<(String, bool)>,
    is_hovered: bool,
    card_index: Option<usize>,
    variant: DashboardCardVariant,
) -> Element<'a, Message> {
    let mut content = column![
        text(title).size(15).font(Font {
            weight: font::Weight::Medium,
            ..Default::default()
        }),
        text(value).size(25).font(Font {
            weight: font::Weight::Medium,
            ..Default::default()
        }),
    ]
    .align_x(Center)
    .spacing(5);

    if let Some((trend_text, is_positive)) = trend {
        let trend_icon = if is_positive {
            icons::arrow_up()
        } else {
            icons::arrow_down()
        };

        let trend_row = container(row![
            svg::Svg::new(trend_icon).width(14).height(14),
            text(trend_text).size(12).font(Font {
                weight: font::Weight::Medium,
                ..Default::default()
            }),
        ])
        .align_bottom(Length::Fill);

        content = content.push(trend_row);
    }

    let card = container(content)
        .height(Length::Fixed(100.0))
        .padding([10, 20])
        .center_x(Length::Fixed(180.0))
        .style(move |theme: &Theme| styles::card_style_with_variant(theme, is_hovered, variant));

    mouse_area(card)
        .on_enter(Message::DashboardCardHovered(card_index))
        .on_exit(Message::DashboardCardHovered(None))
        .into()
}

fn menu_item_container<'a>(
    item: svg::Svg<'a>,
    item_text: &'a str,
    is_item_selected: bool,
    is_item_hovered: bool,
    is_side_menu_hovered: bool,
    animated_container_height: &Animated<bool, Instant>,
    now: Instant,
) -> Container<'a, Message> {
    let create_text = move |is_hovered: bool, is_selected: bool| {
        text(item_text)
            .font(Font {
                weight: font::Weight::Light,
                ..Default::default()
            })
            .size(11)
            .wrapping(text::Wrapping::None)
            .style(move |theme: &Theme| {
                if is_hovered {
                    text::Style {
                        color: Some(Color {
                            r: 0.1,
                            g: 0.1,
                            b: 1.0,
                            a: 0.9,
                        }),
                    }
                } else if is_selected {
                    text::Style {
                        color: Some(theme.extended_palette().background.strong.text),
                    }
                } else {
                    text::Style::default()
                }
            })
    };

    let content = if is_item_hovered || is_side_menu_hovered {
        row![item, create_text(is_item_hovered, is_item_selected)]
            .align_y(Center)
            .spacing(10)
    } else {
        row![item,].spacing(10)
    };

    container(content)
        .width(Length::Fill)
        .align_left(Length::Fill)
        .center_y(Length::Fixed(
            animated_container_height.animate_bool(40.0, 45.0, now),
        ))
        .padding([0, 20])
        .style(move |theme: &Theme| {
            if is_item_selected {
                container::Style {
                    text_color: Some(Color::from_rgba(1.0, 1.0, 1.0, 0.5)),
                    background: Some(Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.5))),
                    ..Default::default()
                }
            } else {
                container::transparent(theme)
            }
        })
}

// =========================================
// CUSTOM COMPONENTS
// =========================================
struct IncomeData {
    potential: f32,
    actual: f32,
    month: String,
}

struct GroupedBarChart {
    data: Vec<IncomeData>,
    cache: canvas::Cache,
}

impl GroupedBarChart {
    fn default(data: Vec<IncomeData>) -> Self {
        Self {
            data,
            cache: canvas::Cache::new(),
        }
    }
}

impl<Message> canvas::Program<Message> for GroupedBarChart {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let max_bar = self
                .data
                .iter()
                .flat_map(|data| [data.potential, data.potential])
                .fold(0.0f32, f32::max);

            let padding = 50.0;
            let chart_width = frame.width() - padding * 2.0;
            let chart_height = frame.height() - padding * 2.0;

            let num_groups = self.data.len();
            let bar_scale = chart_height / (max_bar * 1.1);
            let group_width = chart_width / num_groups as f32;
            let bar_width = group_width * 0.30;
            let gap_between_bars = group_width * 0.1;
            let group_padding = group_width * 0.2;

            draw_axes(frame, padding, chart_width, chart_height);

            for (i, data) in self.data.iter().enumerate() {
                let group_x = padding + (i as f32 * group_width);

                let potential_earnings_x = group_x + group_padding;
                let potential_earnings_bar_height = data.potential * bar_scale;
                let potential_earnings_y = padding + chart_height - potential_earnings_bar_height;

                let potential_earning_bar = Path::rectangle(
                    Point::new(potential_earnings_x, potential_earnings_y),
                    Size::new(bar_width, potential_earnings_bar_height),
                );
                frame.fill(&potential_earning_bar, Color::from_rgb(0.3, 0.6, 0.9));

                let actual_earnings_x = potential_earnings_x + bar_width + gap_between_bars;
                let actual_earnings_bar_height = data.actual * bar_scale;
                let actual_earnings_y = padding + chart_height - actual_earnings_bar_height;

                let actual_earning_bar = Path::rectangle(
                    Point::new(actual_earnings_x, actual_earnings_y),
                    Size::new(bar_width, actual_earnings_bar_height),
                );
                frame.fill(&actual_earning_bar, Color::from_rgba(0.7, 0.7, 0.7, 0.5));

                let label_x = group_x + (group_width / 2.0);
                let label_y = padding + chart_height + 10.0;

                frame.fill_text(Text {
                    content: data.month.clone(),
                    position: Point { x: label_x, y: label_y },
                    color: Color::BLACK,
                    size: 11.0.into(),
                    align_x: iced::advanced::text::Alignment::Center,
                    ..Default::default()

                });
            }
        });
        vec![geometry]
    }
}

struct LineChart {
    data: Vec<Attendance>,
    cache: canvas::Cache,
}

struct Attendance {
    month: String,
    attended_days: i32,
}

impl LineChart {
    fn default(data: Vec<Attendance>) -> Self {
        Self {
            data,
            cache: canvas::Cache::new(),
        }
    }
}

impl<Message> canvas::Program<Message> for LineChart {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::advanced::mouse::Cursor,
    ) -> Vec<canvas::Geometry> {
        let geometry = self.cache.draw(renderer, bounds.size(), |frame| {
            let max_bar = self
                .data
                .iter()
                .map(|dp| dp.attended_days)
                .max()
                .unwrap() as f32;
            let padding = 50.0;
            let chart_width = frame.width() - padding * 2.0;
            let chart_height = frame.height() - padding * 2.0;

            let num_groups = self.data.len();
            let bar_scale = chart_height / (max_bar * 1.1);
            let group_width = chart_width / num_groups as f32;
            let bar_width = group_width * 0.30;
            let gap_between_bars = group_width * 0.1;
            let group_padding = group_width * 0.2;

            draw_axes(frame, padding, chart_width, chart_height);

            for (i, data) in self.data.iter().enumerate() {
            }
        });
        vec![geometry]
    }
}

fn draw_axes(frame: &mut Frame, padding: f32, width: f32, height: f32) {
    // y-axis
    let y_axis = Path::line(
        Point::new(padding, padding),
        Point::new(padding, padding + height),
    );
    frame.stroke(
        &y_axis,
        Stroke::default()
            .with_color(Color::from_rgb(0.5, 0.5, 0.5))
            .with_width(2.0),
    );

    // x-axis
    let x_axis = Path::line(
        Point::new(padding, padding + height),
        Point::new(padding + width, padding + height),
    );
    frame.stroke(
        &x_axis,
        Stroke::default()
            .with_color(Color::from_rgb(0.5, 0.5, 0.5))
            .with_width(2.0),
    );
}

// =========================================
// STYLES
// =========================================

mod styles {
    use super::Background;
    use super::Border;
    use super::Color;
    use super::DashboardCardVariant;
    use super::Shadow;
    use super::Theme;
    use super::Vector;
    use super::container;
    use super::svg;

    pub fn menu_icon_style(is_item_hovered: bool) -> svg::Style {
        if is_item_hovered {
            svg::Style {
                color: Some(Color {
                    r: 0.1,
                    g: 0.1,
                    b: 1.0,
                    a: 0.9,
                }),
            }
        } else {
            svg::Style { color: None }
        }
    }

    pub fn card_style_with_variant(
        theme: &Theme,
        is_hovered: bool,
        variant: DashboardCardVariant,
    ) -> container::Style {
        let palette = theme.extended_palette();

        let background_color = match variant {
            DashboardCardVariant::Attendance => Some(palette.primary.weak.color),
            DashboardCardVariant::ActualEarnings => Some(Color::from_rgba(0.4, 1.0, 0.5, 0.6)),
            DashboardCardVariant::PotentialEarnings => Some(Color::from_rgba(0.8, 0.7, 0.8, 0.4)),
            DashboardCardVariant::RevenueLost => Some(Color::from_rgba(1.0, 0.5, 0.2, 0.6)),
        };

        container::Style {
            background: background_color.map(Background::Color),
            border: Border {
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
    }
}

// =========================================
// ICONS & ASSETS
// =========================================
mod icons {
    use iced::widget::svg;
    use std::sync::OnceLock;

    static PLUS: OnceLock<svg::Handle> = OnceLock::new();
    static EDIT: OnceLock<svg::Handle> = OnceLock::new();
    static CALENDAR: OnceLock<svg::Handle> = OnceLock::new();
    static SCHEDULE: OnceLock<svg::Handle> = OnceLock::new();
    static CHECK_CIRCLE: OnceLock<svg::Handle> = OnceLock::new();
    static PAYMENTS: OnceLock<svg::Handle> = OnceLock::new();
    static DASHBOARD: OnceLock<svg::Handle> = OnceLock::new();
    static ARROW_DOWN: OnceLock<svg::Handle> = OnceLock::new();
    static ARROW_UP: OnceLock<svg::Handle> = OnceLock::new();
    static STUDENT: OnceLock<svg::Handle> = OnceLock::new();
    static LOGO: OnceLock<svg::Handle> = OnceLock::new();
    static LOGO_EXPANDED: OnceLock<svg::Handle> = OnceLock::new();
    static SETTINGS: OnceLock<svg::Handle> = OnceLock::new();
    static LOGOUT: OnceLock<svg::Handle> = OnceLock::new();

    fn icon_path(name: &str) -> String {
        format!("{}/resources/icons/{}", env!("CARGO_MANIFEST_DIR"), name)
    }

    pub fn plus() -> svg::Handle {
        PLUS.get_or_init(|| {
            svg::Handle::from_path(icon_path("add_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg"))
        })
        .clone()
    }

    pub fn edit() -> svg::Handle {
        EDIT.get_or_init(|| svg::Handle::from_path(icon_path("pen-to-square-regular-full.svg")))
            .clone()
    }

    pub fn calendar() -> svg::Handle {
        CALENDAR
            .get_or_init(|| {
                svg::Handle::from_path(icon_path(
                    "calendar_today_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg",
                ))
            })
            .clone()
    }

    pub fn schedule() -> svg::Handle {
        SCHEDULE
            .get_or_init(|| {
                svg::Handle::from_path(icon_path(
                    "schedule_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg",
                ))
            })
            .clone()
    }

    pub fn check_circle() -> svg::Handle {
        CHECK_CIRCLE
            .get_or_init(|| {
                svg::Handle::from_path(icon_path(
                    "check_circle_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg",
                ))
            })
            .clone()
    }

    pub fn payments() -> svg::Handle {
        PAYMENTS
            .get_or_init(|| {
                svg::Handle::from_path(icon_path(
                    "payments_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg",
                ))
            })
            .clone()
    }

    pub fn dashboard() -> svg::Handle {
        DASHBOARD
            .get_or_init(|| {
                svg::Handle::from_path(icon_path(
                    "dashboard_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg",
                ))
            })
            .clone()
    }

    pub fn student_manager() -> svg::Handle {
        STUDENT
            .get_or_init(|| {
                svg::Handle::from_path(icon_path(
                    "school_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg",
                ))
            })
            .clone()
    }

    pub fn arrow_up() -> svg::Handle {
        ARROW_UP
            .get_or_init(|| {
                svg::Handle::from_path(icon_path(
                    "arrow_upward_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg",
                ))
            })
            .clone()
    }

    pub fn arrow_down() -> svg::Handle {
        ARROW_DOWN
            .get_or_init(|| {
                svg::Handle::from_path(icon_path(
                    "arrow_downward_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg",
                ))
            })
            .clone()
    }

    pub fn logo() -> svg::Handle {
        LOGO.get_or_init(|| svg::Handle::from_path(icon_path("nhoma_short_logo.svg")))
            .clone()
    }

    pub fn logo_expanded() -> svg::Handle {
        LOGO_EXPANDED
            .get_or_init(|| svg::Handle::from_path(icon_path("nhoma_logo.svg")))
            .clone()
    }

    pub fn settings() -> svg::Handle {
        SETTINGS
            .get_or_init(|| {
                svg::Handle::from_path(icon_path(
                    "settings_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg",
                ))
            })
            .clone()
    }

    pub fn logout() -> svg::Handle {
        LOGOUT
            .get_or_init(|| {
                svg::Handle::from_path(icon_path(
                    "logout_24dp_1F1F1F_FILL0_wght400_GRAD0_opsz24.svg",
                ))
            })
            .clone()
    }
}

// =========================================
// MOCK DATA & TESTING
// =========================================
#[cfg(debug_assertions)]
fn mock_domain() -> Domain {
    Domain { 
        tutor: Tutor { 
            id: String::from("tutor1"), 
            name: PersonalName { 
                first: String::from("Andy"), 
                last: String::from("Murray"), 
                other: None::<String> 
            } 
        }, 
        students: mock_student_data(), 
        monthly_summaries: mock_monthly_summaries(),
    }
}

fn mock_student_data() -> Vec<Student> {
    vec![
        Student {
            id: String::from("student1"),
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
                payment_type: PaymentType::PerSession,
                amount: 150.0,
            },
        },
        Student {
            id: String::from("student2"),
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
                payment_type: PaymentType::Monthly,
                amount: 150.0,
            },
        },
    ]
}

fn mock_monthly_summaries() -> Vec<MonthlySummary> {
    vec![
        MonthlySummary {
            year_month: YearMonth { year: 2025, month: Month::October },
            actual_revenue: 1500.0,
            potential_revenue: 2000.0,
            total_actual_sessions: 5,
            total_scheduled_sessions: 8,
        },

        MonthlySummary {
            year_month: YearMonth { year: 2025, month: Month::November },
            actual_revenue: 1200.0,
            potential_revenue: 1800.0,
            total_actual_sessions: 3,
            total_scheduled_sessions: 4,
        },
    ]
}

fn mock_income_data() -> Vec<IncomeData> {
    vec![
        IncomeData {
            potential: 2000.0,
            actual: 1500.0,
            month: "Sep".to_string(),
        },
        IncomeData {
            potential: 1300.0,
            actual: 1000.0,
            month: "Oct".to_string(),
        },
        IncomeData {
            potential: 3000.0,
            actual: 2400.0,
            month: "Nov".to_string(),
        },
    ]
}

fn mock_attendance_data() -> Vec<Attendance> {
    vec![
        Attendance {
            month: "Sep".to_string(),
            attended_days: 8,
        },
        Attendance {
            month: "Oct".to_string(),
            attended_days: 3,
        },
        Attendance {
            month: "Nov".to_string(),
            attended_days: 5,
        },
    ]
}

// =========================================
// MAIN
// =========================================
fn main() -> iced::Result {
    iced::application(
        TutoringManager::new,
        TutoringManager::update,
        TutoringManager::view,
    )
    .title(TutoringManager::title)
    .subscription(TutoringManager::subscription)
    .window(iced::window::Settings {
        size: Size::new(1200.0, 800.0), 
        maximized: false, 
        fullscreen: false, 
        min_size: Some(Size::new(900.0, 700.0)), 
        resizable: true, 
        closeable: true, 
        minimizable: true, 
        icon: None, 
        exit_on_close_request: true, 
        ..Default::default()
    })
    .run()
}
