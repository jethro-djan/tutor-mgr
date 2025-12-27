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
static CANCEL: OnceLock<svg::Handle> = OnceLock::new();
static DELETE: OnceLock<svg::Handle> = OnceLock::new();

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

pub fn cancel() -> svg::Handle {
    CANCEL
        .get_or_init(|| svg::Handle::from_path(icon_path("cancel.svg")))
        .clone()
}

pub fn delete() -> svg::Handle {
    DELETE
        .get_or_init(|| svg::Handle::from_path(icon_path("delete.svg")))
        .clone()
}
