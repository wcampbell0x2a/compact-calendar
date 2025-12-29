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
        details: HashMap<NaiveDate, DateDetail>,
        ranges: Vec<DateRange>,
    ) -> Self {
        Calendar {
            year,
            week_start,
            weekend_display,
            color_mode,
            past_date_display,
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
