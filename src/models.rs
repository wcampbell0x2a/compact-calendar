use chrono::{Datelike, NaiveDate};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeekStart {
    Monday,
    Sunday,
}

impl WeekStart {
    pub fn from_sunday_flag(sunday: bool) -> Self {
        if sunday {
            Self::Sunday
        } else {
            Self::Monday
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeekendDisplay {
    Dimmed,
    Normal,
}

impl WeekendDisplay {
    pub fn from_no_dim_flag(no_dim_weekends: bool) -> Self {
        if no_dim_weekends {
            Self::Normal
        } else {
            Self::Dimmed
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Normal,
    Work,
}

impl ColorMode {
    pub fn from_work_flag(work: bool) -> Self {
        if work {
            Self::Work
        } else {
            Self::Normal
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PastDateDisplay {
    Strikethrough,
    Normal,
}

impl PastDateDisplay {
    pub fn from_no_strikethrough_flag(no_strikethrough: bool) -> Self {
        if no_strikethrough {
            Self::Normal
        } else {
            Self::Strikethrough
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonthFilter {
    All,                       // Default: show all months
    Single(u32),               // --month N: show specific month (1-12)
    Current,                   // --month current
    CurrentWithFollowing(u32), // --month current --following-months N
}

impl MonthFilter {
    /// Create MonthFilter from CLI arguments
    pub fn from_cli_args(
        month: Option<&str>,
        following_months: Option<u32>,
    ) -> Result<Self, String> {
        match (month, following_months) {
            (None, None) => Ok(MonthFilter::All),
            (None, Some(_)) => Err("--following-months requires --month current".to_string()),
            (Some(month_str), following) => {
                let base_filter = Self::parse_month(month_str)?;
                Self::apply_following_months(base_filter, following)
            }
        }
    }

    /// Apply following_months modifier to a base filter
    fn apply_following_months(base: Self, following: Option<u32>) -> Result<Self, String> {
        match (base, following) {
            (filter, None) => Ok(filter),
            (MonthFilter::Current, Some(n)) => {
                if n > 11 {
                    Err("--following-months cannot exceed 11".to_string())
                } else {
                    Ok(MonthFilter::CurrentWithFollowing(n))
                }
            }
            (_, Some(_)) => {
                Err("--following-months can only be used with --month current".to_string())
            }
        }
    }

    /// Parse month from string (number, name, or "current")
    fn parse_month(input: &str) -> Result<Self, String> {
        // Check for "current" first
        if input.eq_ignore_ascii_case("current") {
            return Ok(MonthFilter::Current);
        }

        // Try parsing as number
        if let Ok(num) = input.parse::<u32>() {
            return Self::validate_month_number(num);
        }

        // Parse as month name
        Self::parse_month_name(input)
    }

    fn validate_month_number(num: u32) -> Result<Self, String> {
        if (1..=12).contains(&num) {
            Ok(MonthFilter::Single(num))
        } else {
            Err(format!("Month number must be 1-12, got {}", num))
        }
    }

    fn parse_month_name(input: &str) -> Result<Self, String> {
        let month_num = match input.to_lowercase().as_str() {
            "january" | "jan" => 1,
            "february" | "feb" => 2,
            "march" | "mar" => 3,
            "april" | "apr" => 4,
            "may" => 5,
            "june" | "jun" => 6,
            "july" | "jul" => 7,
            "august" | "aug" => 8,
            "september" | "sep" | "sept" => 9,
            "october" | "oct" => 10,
            "november" | "nov" => 11,
            "december" | "dec" => 12,
            _ => {
                return Err(format!(
                    "Invalid month: '{}'. Use 1-12, month name (e.g., 'march'), or 'current'",
                    input
                ))
            }
        };

        Ok(MonthFilter::Single(month_num))
    }

    /// Get the range of months to display (start_month, end_month) for the given year
    pub fn get_month_range(&self, _year: i32) -> (u32, u32) {
        match self {
            MonthFilter::All => (1, 12),
            MonthFilter::Single(m) => (*m, *m),
            MonthFilter::Current => {
                let month = Self::get_current_month_number();
                (month, month)
            }
            MonthFilter::CurrentWithFollowing(n) => {
                let start_month = Self::get_current_month_number();
                let end_month = (start_month + n).min(12);
                (start_month, end_month)
            }
        }
    }

    fn get_current_month_number() -> u32 {
        chrono::Local::now().date_naive().month()
    }

    /// Check if a specific month should be displayed
    pub fn should_display_month(&self, month: u32, year: i32) -> bool {
        let (start, end) = self.get_month_range(year);
        month >= start && month <= end
    }

    /// Get the filtered date range (start_date, end_date) for rendering
    pub fn get_date_range(&self, year: i32) -> (NaiveDate, NaiveDate) {
        let (start_month, end_month) = self.get_month_range(year);

        let start_date = NaiveDate::from_ymd_opt(year, start_month, 1).unwrap();
        let end_date = Self::get_last_day_of_month(year, end_month);

        (start_date, end_date)
    }

    fn get_last_day_of_month(year: i32, month: u32) -> NaiveDate {
        if month == 12 {
            NaiveDate::from_ymd_opt(year, 12, 31).unwrap()
        } else {
            // Get first day of next month, then go back one day
            NaiveDate::from_ymd_opt(year, month + 1, 1)
                .unwrap()
                .pred_opt()
                .unwrap()
        }
    }
}

#[derive(Debug, Clone)]
pub struct DateDetail {
    pub description: String,
    pub color: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
    pub color: String,
    pub description: Option<String>,
}

pub struct Calendar {
    pub year: i32,
    pub week_start: WeekStart,
    pub weekend_display: WeekendDisplay,
    pub color_mode: ColorMode,
    pub past_date_display: PastDateDisplay,
    pub month_filter: MonthFilter,
    pub details: HashMap<NaiveDate, DateDetail>,
    pub ranges: Vec<DateRange>,
}

impl Calendar {
    pub fn new(
        year: i32,
        week_start: WeekStart,
        weekend_display: WeekendDisplay,
        color_mode: ColorMode,
        past_date_display: PastDateDisplay,
        month_filter: MonthFilter,
        details: HashMap<NaiveDate, DateDetail>,
        ranges: Vec<DateRange>,
    ) -> Self {
        Calendar {
            year,
            week_start,
            weekend_display,
            color_mode,
            past_date_display,
            month_filter,
            details,
            ranges,
        }
    }

    pub fn get_weekday_num(&self, date: NaiveDate) -> u32 {
        match self.week_start {
            WeekStart::Monday => date.weekday().num_days_from_monday(),
            WeekStart::Sunday => date.weekday().num_days_from_sunday(),
        }
    }
}
