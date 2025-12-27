use chrono::{Datelike, Local};
use iced::advanced::graphics::core::font;
use iced::alignment::Vertical;
use iced::widget::canvas::{self, Frame, Path, Stroke, Text};
use iced::widget::{Canvas, Column, Grid, column, container, grid, mouse_area, row, svg, text};
use iced::{
    Background, Border, Center, Color, Element, Font, Length, Point, Rectangle, Renderer, Shadow,
    Size, Task, Theme, Vector,
};

use crate::domain::*;
use crate::icons;
use crate::ui_components::{global_content_container, page_header};

pub struct DashboardState {
    hovered_dashboard_card: Option<usize>,
    barchart: GroupedBarChart,
    linechart: LineChart,
    dashboard_summary: DashboardSummary,

    is_ready: bool,
}

impl DashboardState {
    pub fn attach_domain(&mut self, domain: &Domain) {
        let income_data = domain.compute_income_data();
        let attendance_data = domain.compute_attendance_data();

        self.barchart = GroupedBarChart::new(income_data);
        self.linechart = LineChart::new(attendance_data);
        self.dashboard_summary = DashboardSummary::compute_from_domain_state(domain);

        self.is_ready = true;
    }

    pub fn empty() -> Self {
        Self {
            hovered_dashboard_card: None,
            barchart: GroupedBarChart::empty(),
            linechart: LineChart::empty(),
            dashboard_summary: DashboardSummary::empty(),

            is_ready: false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum Msg {
    DashboardCardHovered(Option<usize>),
}

pub fn update(state: &mut DashboardState, msg: Msg) -> Task<Msg> {
    match msg {
        Msg::DashboardCardHovered(card_index) => {
            state.hovered_dashboard_card = card_index;
            Task::none()
        }
    }
}

pub fn view<'a>(state: &'a DashboardState) -> Element<'a, Msg> {
    view_dashboard(state)
}

struct DashboardSummary {
    attendance: AttendanceSummary,
    actual_revenue: ActualRevenueSummary,
    potential_revenue: PotentialRevenueSummary,
    lost_revenue: LostRevenueSummary,
}

impl DashboardSummary {
    fn empty() -> Self {
        Self {
            attendance: AttendanceSummary {
                total_scheduled_sessions: 0,
                total_actual_sessions: 0,
            },
            actual_revenue: ActualRevenueSummary {
                amount: 0.0f32,
                trend: NumberTrend::NoData,
            },
            potential_revenue: PotentialRevenueSummary { amount: 0.0f32 },
            lost_revenue: LostRevenueSummary {
                amount: 0.0f32,
                trend: NumberTrend::NoData,
            },
        }
    }

    fn compute_from_domain_state(domain: &Domain) -> Self {
        let today = Local::now().naive_local().date();
        let current_year = today.year();
        let current_month = today.month();

        let total_actual_sessions = domain
            .students
            .iter()
            .map(|student| {
                compute_monthly_completed_sessions(student, current_month, current_year) as usize
            })
            .sum();

        let total_scheduled_sessions = domain
            .students
            .iter()
            .map(|student| {
                compute_monthly_scheduled_sessions(student, current_month, current_year) as usize
            })
            .sum();

        let potential_earnings = domain
            .students
            .iter()
            .map(|std| {
                compute_monthly_sum(
                    std,
                    current_month,
                    current_year,
                    compute_monthly_scheduled_sessions,
                )
            })
            .sum();

        let actual_earnings = domain
            .students
            .iter()
            .map(|std| {
                compute_monthly_sum(
                    std,
                    current_month,
                    current_year,
                    compute_monthly_completed_sessions,
                )
            })
            .sum();

        let attendance = AttendanceSummary {
            total_actual_sessions,
            total_scheduled_sessions,
        };

        let actual_income_trend = domain.get_actual_income_trend_direction();

        let actual_revenue = ActualRevenueSummary {
            amount: actual_earnings,
            trend: actual_income_trend,
        };
        let potential_revenue = PotentialRevenueSummary {
            amount: potential_earnings,
        };
        let lost_revenue = LostRevenueSummary {
            amount: potential_earnings - actual_earnings,
            trend: NumberTrend::NoData,
        };

        Self {
            attendance,
            actual_revenue,
            potential_revenue,
            lost_revenue,
        }
    }
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

pub struct GroupedBarChart {
    data: Vec<IncomeData>,
    cache: canvas::Cache,
}

impl GroupedBarChart {
    fn new(data: Vec<IncomeData>) -> Self {
        Self {
            data,
            cache: canvas::Cache::new(),
        }
    }

    fn empty() -> Self {
        Self {
            data: Vec::new(),
            cache: canvas::Cache::new(),
        }
    }
}

impl<Msg> canvas::Program<Msg> for GroupedBarChart {
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
            if self.data.is_empty() {
                frame.fill_text(Text {
                    content: "No attendance data yet".into(),
                    position: Point::new(frame.width() / 2.0, frame.height() / 2.0),
                    color: Color::from_rgb(0.5, 0.5, 0.5),
                    size: 14.0.into(),
                    align_x: iced::advanced::text::Alignment::Center,
                    align_y: iced::alignment::Vertical::Center,
                    ..Default::default()
                });
                return;
            }

            let max_bar = self
                .data
                .iter()
                .flat_map(|data| [data.potential, data.potential])
                .fold(0.0f32, f32::max);

            let padding = 20.0;
            let chart_width = frame.width() - padding * 2.0;
            let chart_height = frame.height() - padding * 2.5;

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
                    content: data.month_year.0.clone(),
                    position: Point {
                        x: label_x,
                        y: label_y,
                    },
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

impl LineChart {
    fn new(data: Vec<Attendance>) -> Self {
        Self {
            data,
            cache: canvas::Cache::new(),
        }
    }

    fn empty() -> Self {
        Self {
            data: Vec::new(),
            cache: canvas::Cache::new(),
        }
    }
}

impl<Msg> canvas::Program<Msg> for LineChart {
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
            if self.data.is_empty() {
                frame.fill_text(Text {
                    content: "No income data yet".into(),
                    position: Point::new(frame.width() / 2.0, frame.height() / 2.0),
                    color: Color::from_rgb(0.5, 0.5, 0.5),
                    size: 14.0.into(),
                    align_x: iced::advanced::text::Alignment::Center,
                    align_y: iced::alignment::Vertical::Center,
                    ..Default::default()
                });
                return;
            }

            let max_bar = self.data.iter().map(|dp| dp.attended_days).max().unwrap() as f32;
            let padding = 20.0;
            let chart_width = frame.width() - padding * 2.0;
            let chart_height = frame.height() - padding * 2.5;
            let bar_scale = chart_height / (max_bar * 1.1);

            let num_groups = self.data.len();
            let group_width = chart_width / num_groups as f32;

            // for axes
            draw_axes(frame, padding, chart_width, chart_height);

            let points: Vec<Point> = self
                .data
                .iter()
                .enumerate()
                .map(|(i, dp)| {
                    let data = dp.attended_days as f32;
                    let group_x = padding + (i as f32 * group_width);
                    let income_y_scale = data * bar_scale;

                    let point_x = group_x + (group_width / 2.0);
                    let point_y = padding + chart_height - income_y_scale;

                    Point::new(point_x, point_y)
                })
                .collect();

            // for points
            for point in &points {
                let path = Path::circle(*point, 4.0);
                frame.fill(&path, Color::BLACK);
            }

            // connecting lines
            for window in points.windows(2) {
                let line = Path::line(window[0], window[1]);
                frame.stroke(
                    &line,
                    Stroke::default().with_color(Color::BLACK).with_width(1.5),
                );
            }

            // for labels
            for (i, data) in self.data.iter().enumerate() {
                let group_x = padding + (i as f32 * group_width);

                let label_x = group_x + (group_width / 2.0);
                let label_y = padding + chart_height + 10.0;

                frame.fill_text(Text {
                    content: data.month.clone(),
                    position: Point {
                        x: label_x,
                        y: label_y,
                    },
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

fn view_dashboard(state: &DashboardState) -> Element<'_, Msg> {
    struct CardInfo {
        title: String,
        value: String,
        trend: Option<(String, Option<bool>)>,
        hovered_dashboard: Option<usize>,
        variant: DashboardCardVariant,
    }

    let summary = &state.dashboard_summary;

    let attendance_rate = if summary.attendance.total_scheduled_sessions > 0 {
        format!(
            "{:.0}%",
            summary.attendance.total_actual_sessions as f32
                / summary.attendance.total_scheduled_sessions as f32
                * 100.0
        )
    } else {
        "--".to_string()
    };

    let trend_format = |trend: &NumberTrend| -> (String, Option<bool>) {
        match trend {
            NumberTrend::NoData => (format!("{:.1}%", 0.0), None),
            NumberTrend::Trend {
                trend_direction,
                percentage_change,
            } => match trend_direction {
                TrendDirection::Up => (format!("{:.1}%", percentage_change), Some(true)),
                TrendDirection::Down => (format!("{:.1}%", percentage_change), Some(true)),
            },
        }
    };

    let card_data = [
        CardInfo {
            title: "Attendance Rate".into(),
            value: attendance_rate,
            trend: Some(trend_format(&summary.actual_revenue.trend)),
            hovered_dashboard: state.hovered_dashboard_card,
            variant: DashboardCardVariant::Attendance,
        },
        CardInfo {
            title: "Actual Earnings".into(),
            value: format!("GHS {:.2}", summary.actual_revenue.amount),
            trend: Some(trend_format(&summary.actual_revenue.trend)),
            hovered_dashboard: state.hovered_dashboard_card,
            variant: DashboardCardVariant::ActualEarnings,
        },
        CardInfo {
            title: "Potential Earnings".into(),
            value: format!("GHS {:.2}", summary.potential_revenue.amount),
            trend: None,
            hovered_dashboard: state.hovered_dashboard_card,
            variant: DashboardCardVariant::PotentialEarnings,
        },
        CardInfo {
            title: "Revenue Lost".into(),
            value: format!("GHS {:.2}", summary.lost_revenue.amount),
            trend: None,
            hovered_dashboard: state.hovered_dashboard_card,
            variant: DashboardCardVariant::RevenueLost,
        },
    ];

    let summary_section_title = text("Summary").size(14).font(Font {
        weight: font::Weight::Medium,
        ..Default::default()
    });

    let summary_cards_row = grid(card_data.iter().enumerate().map(|(index, card)| {
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
    .columns(4)
    .width(800)
    .height(Length::Fixed(100.0))
    .spacing(16);

    let summary_section = column![
        summary_section_title,
        container(summary_cards_row).align_x(Center).max_width(900),
    ]
    .spacing(12);

    let attendance_trend_chart = view_trend_chart(state);
    let potential_vs_actual_chart = view_grouped_chart(state);

    let graphs_section_title = text("Analytics").size(14).font(Font {
        weight: font::Weight::Medium,
        ..Default::default()
    });
    let graphs = Grid::new()
        .push(attendance_trend_chart)
        .push(potential_vs_actual_chart)
        .columns(3)
        .height(Length::Fixed(300.0))
        .width(1300)
        .spacing(16);

    let graph_section = column![graphs_section_title, graphs,].spacing(12);

    let content = global_content_container(
        Column::new()
            .spacing(40)
            .push(summary_section)
            .push(graph_section),
    )
    .width(Length::Fill)
    .height(Length::Fill);

    let content_with_header = column![page_header("Dashboard"), content,];

    content_with_header.into()
}

fn view_trend_chart(state: &DashboardState) -> Element<'_, Msg> {
    let chart = Canvas::new(&state.linechart)
        .width(Length::Fill)
        .height(Length::Fill);

    container(column![
        container(text!("Attendance Rate").size(20)).center_x(Length::Fill),
        chart
    ])
    // .width(Length::FillPortion(2))
    // .height(Length::Fixed(400.0))
    .padding(20)
    .style(|theme: &Theme| {
        let palette = theme.extended_palette();

        container::Style {
            background: Some(palette.background.weak.color.into()),
            ..Default::default()
        }
    })
    .into()
}

fn view_grouped_chart(state: &DashboardState) -> Element<'_, Msg> {
    let chart = Canvas::new(&state.barchart)
        .width(Length::Fill)
        .height(Length::Fill);

    container(column![
        container(text!("Actual vs Potential Earnings").size(20)).center_x(Length::Fill),
        chart
    ])
    // .width(Length::FillPortion(3))
    // .height(Length::Fixed(400.0))
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
    trend: Option<(String, Option<bool>)>,
    is_hovered: bool,
    card_index: Option<usize>,
    variant: DashboardCardVariant,
) -> Element<'a, Msg> {
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

    if let Some((trend_text, is_positive_opt)) = trend {
        let trend_icon: Option<svg::Handle> = match is_positive_opt {
            None => None,
            Some(true) => Some(icons::arrow_up()),
            Some(false) => Some(icons::arrow_down()),
        };

        let trend_row = match trend_icon {
            None => container(text(trend_text).size(12).font(Font {
                weight: font::Weight::Medium,
                ..Default::default()
            })),
            Some(icon) => container(row![
                svg::Svg::new(icon).width(14).height(14),
                text(trend_text).size(12).font(Font {
                    weight: font::Weight::Medium,
                    ..Default::default()
                }),
            ]),
        }
        .align_bottom(Length::Fill);

        content = content.push(trend_row);
    }

    let card = container(content)
        .height(Length::Fixed(100.0))
        .padding([10, 20])
        .center_x(Length::Fixed(180.0))
        .style(move |theme: &Theme| card_style_with_variant(theme, is_hovered, variant));

    mouse_area(card)
        .on_enter(Msg::DashboardCardHovered(card_index))
        .on_exit(Msg::DashboardCardHovered(None))
        .into()
}

fn card_style_with_variant(
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
