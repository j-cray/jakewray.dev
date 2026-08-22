use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ComposerDraftData {
    pub title: String,
    pub slug: String,
    pub images: Vec<String>,
    pub caption: String,
    pub display_date: String,
    pub byline: String,
    pub content: String,
    pub updated_at: String,
}

pub fn current_date_string() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let date = js_sys::Date::new_0();
        let months = [
            "January",
            "February",
            "March",
            "April",
            "May",
            "June",
            "July",
            "August",
            "September",
            "October",
            "November",
            "December",
        ];
        let month = months[date.get_month() as usize % 12];
        let day = date.get_date();
        let year = date.get_full_year();
        format!("{} {}, {}", month, day, year)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        "July 27, 2026".to_string()
    }
}

pub fn current_iso_datetime_local() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let date = js_sys::Date::new_0();
        let y = date.get_full_year();
        let m = date.get_month() + 1;
        let d = date.get_date();
        let h = date.get_hours();
        let min = date.get_minutes();
        format!("{:04}-{:02}-{:02}T{:02}:{:02}", y, m, d, h, min)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        "2026-07-28T18:00".to_string()
    }
}

#[allow(dead_code)]
pub fn get_current_time_string() -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let date = js_sys::Date::new_0();
        let h = date.get_hours();
        let m = date.get_minutes();
        let s = date.get_seconds();
        format!("{:02}:{:02}:{:02}", h, m, s)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        "18:00:00".to_string()
    }
}
