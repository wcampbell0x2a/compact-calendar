use chrono::{Datelike, NaiveDate};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeekStart {
    Monday,
    Sunday,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeekendDisplay {
    Dimmed,
    Normal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorMode {
    Normal,
    Work,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PastDateDisplay {
    Strikethrough,
    Normal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonthFilter {
    All,                       // Default: show all months
    Single(u32),               // --month N: show specific month (1-12)
    Current,                   // --month current
    CurrentWithFollowing(u32), // --month current --following-months N
}

impl MonthFilter {
    /// Parse month from string (number, name, or "current")
    pub fn parse_month(input: &str) -> Result<Self, String> {
        // Check for "current" first
        if input.eq_ignore_ascii_case("current") {
            return Ok(MonthFilter::Current);
        }

        // Try parsing as number
        if let Ok(num) = input.parse::<u32>() {
            if num >= 1 && num <= 12 {
                return Ok(MonthFilter::Single(num));
            }
            return Err(format!("Month number must be 1-12, got {}", num));
        }

        // Parse as month name (case-insensitive)
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
    pub fn get_month_range(&self, year: i32) -> (u32, u32) {
        match self {
            MonthFilter::All => (1, 12),
            MonthFilter::Single(m) => (*m, *m),
            MonthFilter::Current => {
                let today = chrono::Local::now().date_naive();
                if year == today.year() {
                    (today.month(), today.month())
                } else {
                    // Use the same month number from today's date in the specified year
                    (today.month(), today.month())
                }
            }
            MonthFilter::CurrentWithFollowing(n) => {
                let today = chrono::Local::now().date_naive();
                let start_month = if year == today.year() {
                    today.month()
                } else {
                    today.month()
                };
                let end_month = (start_month + n).min(12);
                (start_month, end_month)
            }
        }
    }

    /// Check if a specific month should be displayed
    pub fn should_display_month(&self, month: u32, year: i32) -> bool {
        let (start, end) = self.get_month_range(year);
        month >= start && month <= end
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
