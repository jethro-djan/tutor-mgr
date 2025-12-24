use std::collections::BTreeMap;
use chrono::{DateTime, Datelike, Duration, Local, Month, NaiveDate, TimeZone, Weekday};

#[derive(Debug)]
pub struct Domain {
    pub tutor: Tutor,
    pub students: Vec<Student>,
    // monthly_summaries: Vec<MonthlySummary>,
}

impl Domain {
    pub fn load_state_from_db() -> Self {
        mock_domain()
    }

    // pub fn compute_trend_history(&self) -> Vec<TrendData> {
    //     compute_trend_history_internal(&self.monthly_summaries)
    // }

    pub fn compute_income_data(&self) -> Vec<IncomeData> {
        let students = &self.students;

        let mut students_grouped_by_month: BTreeMap<(u32, i32), Vec<&Student>> = BTreeMap::new();

        for student in students.iter() {
            let student_months: Vec<(u32, i32)> = student
                .actual_sessions
                .iter()
                .map(|dt| (dt.month(), dt.year()))
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            for month_key in student_months {
                students_grouped_by_month
                    .entry(month_key)
                    .or_default()
                    .push(student);
            }
        }

        let income_data: Vec<IncomeData> = students_grouped_by_month
            .iter()
            .map(|(&(m, y), stds)| {
                let actual = stds
                    .iter()
                    .map(|std| compute_monthly_sum(std, m, y, compute_monthly_completed_sessions))
                    .sum();

                let potential = stds
                    .iter()
                    .map(|std| compute_monthly_sum(std, m, y, compute_monthly_scheduled_sessions))
                    .sum();

                let date = NaiveDate::from_ymd_opt(y, m, 1).expect("Invalid date construction");
                let month = date.format("%b").to_string();
                let month_year = (month, y);

                IncomeData {
                    actual,
                    potential,
                    month_year,
                }
            })
            .collect();

        println!("{:#?}", income_data);
        income_data
    }

    pub fn compute_attendance_data(&self) -> Vec<Attendance> {
        let students = &self.students;

        let mut students_grouped_by_month: BTreeMap<(u32, i32), Vec<&Student>> = BTreeMap::new();

        for student in students.iter() {
            let student_months: Vec<(u32, i32)> = student
                .actual_sessions
                .iter()
                .map(|dt| (dt.month(), dt.year()))
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            for month_key in student_months {
                students_grouped_by_month
                    .entry(month_key)
                    .or_default()
                    .push(student);
            }
        }

        let attendance_data: Vec<Attendance> = students_grouped_by_month
            .iter()
            .map(|(&(m, y), stds)| {
                let attended_days =
                    stds.iter().fold(0, |acc, &std| std.actual_sessions.len()) as i32;
                // .map(|std| std.actual_sessions.len())

                let date = NaiveDate::from_ymd_opt(y, m, 1).expect("Invalid date construction");
                let month = date.format("%b").to_string();

                Attendance {
                    attended_days,
                    month,
                }
            })
            .collect();

        attendance_data
    }

    pub fn get_actual_income_trend_direction(&self) -> NumberTrend {
        let income_data = self.compute_income_data();
        if income_data.len() < 2 {
            return compute_trend(0.0, income_data[0].actual);
        }

        let now = Local::now();
        let current_month = now.month();
        let current_year = now.year();

        let prev_month = if current_month == 1 {
            12
        } else {
            current_month - 1
        };
        let prev_year = current_year - 1;

        let month_year_ctr = |month: u32, year: i32| {
            let date = NaiveDate::from_ymd_opt(year, month, 1).expect("Invalid date construction");
            let short_month = date.format("%b").to_string();
            (short_month, year)
        };

        let prev_month_year = month_year_ctr(prev_month, prev_year);

        let current_month_year = month_year_ctr(current_month, current_year);

        let rel_income_data: Vec<&IncomeData> = income_data
            .iter()
            .filter(|data| {
                data.month_year == prev_month_year || data.month_year == current_month_year
            })
            .collect();

        compute_trend(rel_income_data[0].actual, rel_income_data[1].actual)
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
pub struct TrendData {
    pub revenue_trend: ActualRevenueTrendData,
    pub sessions_trend: ActualSessionTrendData,
}

#[derive(Clone)]
pub struct ActualRevenueTrendData {
    pub trend: NumberTrend,
    pub current_revenue: f32,
    pub previous_revenue: f32,
    pub year_month: YearMonth,
}

#[derive(Clone)]
pub struct ActualSessionTrendData {
    pub trend: NumberTrend,
    pub current_sessions: f32,
    pub previous_sessions: f32,
    pub year_month: YearMonth,
}

pub type TrendHistory = Vec<TrendData>;

#[derive(Copy, Clone)]
pub struct YearMonth {
    pub year: i32,
    pub month: Month,
}

#[derive(Debug)]
pub struct Student {
    pub id: String,
    pub name: PersonalName,
    pub subject: TutorSubject,
    pub tabled_sessions: Vec<SessionData>,
    pub actual_sessions: Vec<DateTime<Local>>,

    pub payment_data: PaymentData,
    pub tution_start_date: DateTime<Local>,
}

#[derive(Debug)]
pub struct Tutor {
    pub id: String,
    pub name: PersonalName,
}

#[derive(Debug)]
pub struct PersonalName {
    pub first: String,
    pub last: String,
    pub other: Option<String>,
}

#[derive(Debug)]
pub struct SessionData {
    pub day: Weekday,
    pub time: String,
}

#[derive(Debug)]
pub enum TutorSubject {
    AdditionalMathematics,
    ExtendedMathematics,
    Statistics,
}

impl TutorSubject {
    pub fn as_str(&self) -> &str {
        match self {
            TutorSubject::AdditionalMathematics => "Additional Mathematics",
            TutorSubject::ExtendedMathematics => "Extended Mathematics",
            TutorSubject::Statistics => "Statistics",
        }
    }
}

#[derive(Clone, Debug)]
pub struct PaymentData {
    pub payment_type: PaymentType,
    pub amount: f32,
}

#[derive(Clone, Debug)]
pub enum PaymentType {
    PerSession,
    Monthly,
}

pub fn compute_monthly_sum(
    student: &Student,
    month: u32,
    year: i32,
    compute_sessions_fn: fn(&Student, u32, i32) -> i32,
) -> f32 {
    match student.payment_data.payment_type {
        PaymentType::PerSession => {
            let no_of_days = compute_sessions_fn(student, month, year);
            student.payment_data.amount * (no_of_days as f32)
        }
        // TODO: Logic for actual monthly payment taken vs agreed
        // Maybe based on targets or missed sessions and
        // deductions are per contract
        PaymentType::Monthly => student.payment_data.amount,
    }
}

fn get_month_date_range(year: i32, month: u32) -> (NaiveDate, NaiveDate) {
    let month_start = NaiveDate::from_ymd_opt(year, month, 1).unwrap();
    let month_end = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1).unwrap()
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1).unwrap()
    } - Duration::days(1);

    (month_start, month_end)
}

fn get_all_dates_in_month(year: i32, month: u32) -> Vec<NaiveDate> {
    let (month_start, month_end) = get_month_date_range(year, month);
    let duration = month_end.signed_duration_since(month_start);

    (0..=duration.num_days())
        .map(|i| month_start + Duration::days(i))
        .collect()
}

fn get_scheduled_weekdays(student: &Student) -> Vec<Weekday> {
    student
        .tabled_sessions
        .iter()
        .map(|session| session.day)
        .collect()
}

pub fn compute_monthly_scheduled_sessions(student: &Student, month: u32, year: i32) -> i32 {
    let all_dates = get_all_dates_in_month(year, month);
    let session_days = get_scheduled_weekdays(student);

    all_dates
        .iter()
        .filter(|date| session_days.contains(&date.weekday()))
        .count() as i32
}

pub fn compute_monthly_completed_sessions(student: &Student, month: u32, year: i32) -> i32 {
    let (month_start, month_end) = get_month_date_range(year, month);
    let session_days = get_scheduled_weekdays(student);

    let actual_session_dates: Vec<NaiveDate> = student
        .actual_sessions
        .iter()
        .map(|dt| dt.naive_local().date())
        .filter(|date| date >= &month_start && date <= &month_end)
        .collect();

    actual_session_dates
        .iter()
        .filter(|date| session_days.contains(&date.weekday()))
        .count() as i32
}

pub fn get_next_session(student: &Student) -> NaiveDate {
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
        return Vec::<TrendData>::new();
    }

    let mut sorted_summaries = monthly_summaries.to_vec();
    sorted_summaries.sort_by_key(|summary| summary.year_month.month);

    let mut revenue_trend = Vec::new();
    let mut sessions_trend = Vec::new();

    let mut trend_history = Vec::new();

    for i in 1..monthly_summaries.len() {
        let previous = monthly_summaries[i - 1];
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
                trend: compute_trend(
                    previous.total_actual_sessions as f32,
                    current.total_scheduled_sessions as f32,
                ),
                current_sessions: current.total_actual_sessions as f32,
                previous_sessions: previous.total_actual_sessions as f32,
                year_month: current.year_month,
            },
        })
    }

    trend_history
}

pub fn compute_trend(previous: f32, current: f32) -> NumberTrend {
    if previous == 0.0 {
        NumberTrend::NoData
    } else {
        let percentage_change = ((current - previous) / previous * 100.0).abs();
        NumberTrend::Trend {
            trend_direction: if current >= previous {
                TrendDirection::Up
            } else {
                TrendDirection::Down
            },
            percentage_change,
        }
    }
}


#[derive(Clone)]
pub enum NumberTrend {
    NoData,
    Trend {
        trend_direction: TrendDirection,
        percentage_change: f32,
    },
}

#[derive(Clone)]
pub enum TrendDirection {
    Up,
    Down,
}

pub struct Attendance {
    pub month: String,
    pub attended_days: i32,
}

#[derive(Debug)]
pub struct IncomeData {
    pub potential: f32,
    pub actual: f32,
    pub month_year: (String, i32),
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
                other: None::<String>,
            },
        },
        students: mock_student_data(),
        // monthly_summaries: mock_monthly_summaries(),
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

            tution_start_date: Local.with_ymd_and_hms(2025, 11, 1, 00, 00, 00).unwrap(),
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
                payment_type: PaymentType::PerSession,
                amount: 150.0,
            },

            tution_start_date: Local.with_ymd_and_hms(2025, 11, 1, 00, 00, 00).unwrap(),
        },
    ]
}

fn mock_monthly_summaries() -> Vec<MonthlySummary> {
    vec![
        MonthlySummary {
            year_month: YearMonth {
                year: 2025,
                month: Month::October,
            },
            actual_revenue: 1500.0,
            potential_revenue: 2000.0,
            total_actual_sessions: 5,
            total_scheduled_sessions: 8,
        },
        MonthlySummary {
            year_month: YearMonth {
                year: 2025,
                month: Month::November,
            },
            actual_revenue: 1200.0,
            potential_revenue: 1800.0,
            total_actual_sessions: 3,
            total_scheduled_sessions: 4,
        },
    ]
}
