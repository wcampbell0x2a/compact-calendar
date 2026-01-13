use crate::formatting::{MonthInfo, WeekLayout};
use crate::models::{Calendar, ColorMode, DateDetail, PastDateDisplay, WeekStart, WeekendDisplay};
use anstyle::{AnsiColor, Color, Effects, RgbColor, Style};
use chrono::Weekday;
use chrono::{Datelike, NaiveDate};

#[derive(Debug, Clone)]
pub struct RenderState {
    pub week_num: i32,
    pub current_month: Option<u32>,
    pub is_first_month: bool,
    pub current_date: NaiveDate,
}

impl RenderState {
    pub fn new(start_date: NaiveDate) -> Self {
        Self {
            week_num: 1,
            current_month: None,
            is_first_month: true,
            current_date: start_date,
        }
    }

    pub fn advance_week(&mut self, days: i64) {
        self.week_num += 1;
        self.current_date = self
            .current_date
            .checked_add_signed(chrono::Duration::days(days))
            .unwrap();
    }

    pub fn set_current_month(&mut self, month: u32) {
        self.current_month = Some(month);
        self.is_first_month = false;
    }
}

#[derive(Debug, Clone)]
pub struct WeekRenderContext<'a> {
    pub layout: &'a WeekLayout,
    pub next_layout: Option<&'a WeekLayout>,
    pub week_num: i32,
    pub current_month: Option<u32>,
    pub is_last_week: bool,
}

impl<'a> WeekRenderContext<'a> {
    pub fn new(
        layout: &'a WeekLayout,
        next_layout: Option<&'a WeekLayout>,
        week_num: i32,
        current_month: Option<u32>,
        is_last_week: bool,
    ) -> Self {
        Self {
            layout,
            next_layout,
            week_num,
            current_month,
            is_last_week,
        }
    }

    pub fn get_month_name(&self) -> &'static str {
        if let Some((_, month)) = self.layout.month_start_idx {
            MonthInfo::from_month(month).name
        } else {
            ""
        }
    }

    pub fn has_month_boundary_at(&self, idx: usize) -> bool {
        if idx > 0 && idx < self.layout.dates.len() {
            let prev_date = self.layout.dates[idx - 1];
            let date = self.layout.dates[idx];
            date.month() != prev_date.month() || date.year() != prev_date.year()
        } else {
            false
        }
    }

    pub fn next_is_boundary_after(&self, idx: usize) -> bool {
        if idx < 6 {
            let date = self.layout.dates[idx];
            let next_date = self.layout.dates[idx + 1];
            date.month() != next_date.month() || date.year() != next_date.year()
        } else {
            false
        }
    }
}

#[derive(Debug, Clone)]
pub struct DateStyle {
    pub color: Option<String>,
    pub is_today: bool,
    pub is_past: bool,
    pub is_weekend: bool,
    pub effects: Effects,
}

impl DateStyle {
    pub fn new(
        date: NaiveDate,
        color: Option<String>,
        today: NaiveDate,
        past_date_display: PastDateDisplay,
        weekend_display: WeekendDisplay,
    ) -> Self {
        let is_today = date == today;
        let is_past = past_date_display == PastDateDisplay::Strikethrough && date < today;
        let is_weekend = weekend_display == WeekendDisplay::Dimmed
            && (date.weekday() == Weekday::Sat || date.weekday() == Weekday::Sun);

        let mut effects = Effects::new();
        if is_past {
            effects |= Effects::STRIKETHROUGH;
        }
        if is_today {
            effects |= Effects::UNDERLINE;
        }
        if is_weekend && color.is_none() {
            effects |= Effects::DIMMED;
        }

        Self {
            color,
            is_today,
            is_past,
            is_weekend,
            effects,
        }
    }

    pub fn to_style(&self) -> Style {
        if let Some(ref color) = self.color {
            let base = if self.is_weekend {
                ColorCodes::get_dimmed_bg_color(color)
            } else {
                ColorCodes::get_bg_color(color)
            };
            base.fg_color(ColorCodes::black_text().get_fg_color())
                .effects(self.effects)
        } else {
            Style::new().effects(self.effects)
        }
    }
}

#[derive(Debug, Clone)]
pub struct BorderContext {
    pub month_boundary_idx: Option<usize>,
    pub first_bar_idx: Option<usize>,
}

impl BorderContext {
    pub fn from_layout(layout: &WeekLayout, current_month: Option<u32>, year: i32) -> Self {
        let month_boundary_idx = Self::find_month_boundary(layout);
        let first_bar_idx = Self::find_first_bar(layout, current_month, year);

        Self {
            month_boundary_idx,
            first_bar_idx,
        }
    }

    fn find_month_boundary(layout: &WeekLayout) -> Option<usize> {
        for (idx, &date) in layout.dates.iter().enumerate() {
            if idx > 0 {
                let prev_date = layout.dates[idx - 1];
                if date.month() != prev_date.month() || date.year() != prev_date.year() {
                    return Some(idx);
                }
            }
        }
        None
    }

    fn find_first_bar(layout: &WeekLayout, current_month: Option<u32>, year: i32) -> Option<usize> {
        for (idx, &date) in layout.dates.iter().enumerate() {
            let in_month = date.year() == year && Some(date.month()) == current_month;
            let prev_in_month = if idx > 0 {
                let prev_date = layout.dates[idx - 1];
                prev_date.year() == year && Some(prev_date.month()) == current_month
            } else {
                false
            };

            if in_month && !prev_in_month {
                return Some(idx);
            }
        }
        None
    }

    pub fn calculate_dashes_before(&self, boundary_idx: usize) -> usize {
        (boundary_idx - 1) * 5 + 4
    }

    pub fn calculate_dashes_after(&self, boundary_idx: usize, days_in_week: usize) -> usize {
        (days_in_week - boundary_idx) * 5 - 1
    }
}

#[derive(Debug, Clone, Default)]
pub struct AnnotationContext {
    pub details_queue: Vec<(NaiveDate, DateDetail)>,
    pub shown_ranges: Vec<usize>,
}

impl AnnotationContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_detail(&mut self, date: NaiveDate, detail: DateDetail) {
        if !self.details_queue.iter().any(|(d, _)| d == &date) {
            self.details_queue.push((date, detail));
        }
    }

    pub fn pop_next_detail(&mut self) -> Option<(NaiveDate, DateDetail)> {
        if !self.details_queue.is_empty() {
            Some(self.details_queue.remove(0))
        } else {
            None
        }
    }

    pub fn mark_range_shown(&mut self, idx: usize) {
        if !self.shown_ranges.contains(&idx) {
            self.shown_ranges.push(idx);
        }
    }

    pub fn is_range_shown(&self, idx: usize) -> bool {
        self.shown_ranges.contains(&idx)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ColorValue {
    pub normal: RgbColor,
    pub dimmed: RgbColor,
}

impl ColorValue {
    pub const fn new(normal: RgbColor, dimmed: RgbColor) -> Self {
        Self { normal, dimmed }
    }

    pub fn get_normal_style(&self) -> Style {
        Style::new().bg_color(Some(Color::Rgb(self.normal)))
    }

    pub fn get_dimmed_style(&self) -> Style {
        Style::new().bg_color(Some(Color::Rgb(self.dimmed)))
    }
}

#[derive(Debug, Clone)]
pub struct ColorPalette {
    colors_enabled: bool,
}

impl Default for ColorPalette {
    fn default() -> Self {
        Self {
            colors_enabled: !Self::is_color_disabled(),
        }
    }
}

impl ColorPalette {
    pub fn new() -> Self {
        Self::default()
    }

    fn is_color_disabled() -> bool {
        std::env::var("NO_COLOR").is_ok()
    }

    pub fn are_colors_enabled(&self) -> bool {
        self.colors_enabled
    }

    pub fn get_color_value(name: &str) -> Option<ColorValue> {
        match name {
            "orange" => Some(ColorValue::new(
                RgbColor(255, 143, 64),
                RgbColor(178, 100, 45),
            )),
            "yellow" => Some(ColorValue::new(
                RgbColor(230, 180, 80),
                RgbColor(161, 126, 56),
            )),
            "green" => Some(ColorValue::new(
                RgbColor(170, 217, 76),
                RgbColor(119, 152, 53),
            )),
            "blue" => Some(ColorValue::new(
                RgbColor(89, 194, 255),
                RgbColor(62, 136, 179),
            )),
            "purple" => Some(ColorValue::new(
                RgbColor(210, 166, 255),
                RgbColor(147, 116, 179),
            )),
            "red" => Some(ColorValue::new(
                RgbColor(240, 113, 120),
                RgbColor(168, 79, 84),
            )),
            "cyan" => Some(ColorValue::new(
                RgbColor(149, 230, 203),
                RgbColor(104, 161, 142),
            )),
            "gray" => Some(ColorValue::new(RgbColor(95, 99, 110), RgbColor(67, 69, 77))),
            "light_orange" => Some(ColorValue::new(
                RgbColor(255, 180, 84),
                RgbColor(179, 126, 59),
            )),
            "light_yellow" => Some(ColorValue::new(
                RgbColor(249, 175, 79),
                RgbColor(174, 123, 55),
            )),
            "light_green" => Some(ColorValue::new(
                RgbColor(145, 179, 98),
                RgbColor(102, 125, 69),
            )),
            "light_blue" => Some(ColorValue::new(
                RgbColor(83, 189, 250),
                RgbColor(58, 132, 175),
            )),
            "light_purple" => Some(ColorValue::new(
                RgbColor(210, 166, 255),
                RgbColor(147, 116, 179),
            )),
            "light_red" => Some(ColorValue::new(
                RgbColor(234, 108, 115),
                RgbColor(164, 76, 81),
            )),
            "light_cyan" => Some(ColorValue::new(
                RgbColor(144, 225, 198),
                RgbColor(101, 158, 139),
            )),
            _ => None,
        }
    }

    pub fn get_style(&self, color_name: &str, dimmed: bool) -> Style {
        if !self.colors_enabled {
            return Style::new();
        }

        if let Some(color_value) = Self::get_color_value(color_name) {
            if dimmed {
                color_value.get_dimmed_style()
            } else {
                color_value.get_normal_style()
            }
        } else {
            Style::new()
        }
    }

    pub fn black_text() -> Style {
        Style::new().fg_color(Some(Color::Ansi(AnsiColor::Black)))
    }
}

struct ColorCodes;

impl ColorCodes {
    fn is_color_disabled() -> bool {
        std::env::var("NO_COLOR").is_ok()
    }

    fn get_bg_color(color: &str) -> Style {
        if Self::is_color_disabled() {
            return Style::new();
        }
        let palette = ColorPalette::new();
        palette.get_style(color, false)
    }

    fn get_dimmed_bg_color(color: &str) -> Style {
        if Self::is_color_disabled() {
            return Style::new();
        }
        let palette = ColorPalette::new();
        palette.get_style(color, true)
    }

    fn black_text() -> Style {
        ColorPalette::black_text()
    }

    fn underline() -> Effects {
        Effects::UNDERLINE
    }

    fn strikethrough() -> Effects {
        Effects::STRIKETHROUGH
    }

    fn dim() -> Effects {
        Effects::DIMMED
    }
}

const DAYS_IN_WEEK: usize = 7;
const CALENDAR_WIDTH: usize = 34;
const HEADER_WIDTH: usize = 48;

pub struct CalendarRenderer<'a> {
    calendar: &'a Calendar,
}

impl<'a> CalendarRenderer<'a> {
    pub fn new(calendar: &'a Calendar) -> Self {
        CalendarRenderer { calendar }
    }

    pub fn render(&self) {
        self.print_header();
        self.print_weeks();
        println!();
    }

    pub fn render_to_string(&self) -> String {
        let mut output = String::new();

        let prev_no_color = std::env::var("NO_COLOR").ok();
        std::env::set_var("NO_COLOR", "1");

        output.push_str(&self.header_to_string());
        output.push_str(&self.weeks_to_string());
        output.push('\n');

        match prev_no_color {
            Some(val) => std::env::set_var("NO_COLOR", val),
            None => std::env::remove_var("NO_COLOR"),
        }

        output
    }

    /// Check if a week should be rendered based on month filter
    fn should_render_week(&self, layout: &WeekLayout) -> bool {
        // Include week if ANY of its 7 days fall within the filtered month range
        layout.dates.iter().any(|date| {
            if date.year() != self.calendar.year {
                false
            } else {
                self.calendar
                    .month_filter
                    .should_display_month(date.month(), self.calendar.year)
            }
        })
    }

    /// Get the filtered date range based on month filter
    fn get_filtered_date_range(&self) -> (NaiveDate, NaiveDate) {
        let (start_month, end_month) = self
            .calendar
            .month_filter
            .get_month_range(self.calendar.year);

        let start_date = NaiveDate::from_ymd_opt(self.calendar.year, start_month, 1).unwrap();

        // Calculate the number of days in the end month
        let end_day = if end_month == 12 {
            31
        } else {
            NaiveDate::from_ymd_opt(self.calendar.year, end_month + 1, 1)
                .unwrap()
                .pred_opt()
                .unwrap()
                .day()
        };
        let end_date = NaiveDate::from_ymd_opt(self.calendar.year, end_month, end_day).unwrap();

        (start_date, end_date)
    }

    fn header_to_string(&self) -> String {
        let mut output = String::new();
        output.push_str(&format!("┌{:─<width$}┐\n", "", width = HEADER_WIDTH));

        // Center the title
        let title = format!("COMPACT CALENDAR {}", self.calendar.year);
        output.push_str(&format!("│{:^width$}│\n", title, width = HEADER_WIDTH));

        output.push_str(&format!("├{:─<width$}┤\n", "", width = HEADER_WIDTH));
        output.push_str("│              ");
        match self.calendar.week_start {
            WeekStart::Monday => output.push_str("Mon  Tue  Wed  Thu  Fri  Sat  Sun │\n"),
            WeekStart::Sunday => output.push_str("Sun  Mon  Tue  Wed  Thu  Fri  Sat │\n"),
        }
        output
    }

    fn weeks_to_string(&self) -> String {
        let mut output = String::new();
        let (start_date, end_date) = self.get_filtered_date_range();

        let mut current_date = self.align_to_week_start(start_date);
        let mut week_num = 1;
        let mut current_month: Option<u32> = None;

        let mut details_queue: Vec<(NaiveDate, DateDetail)> = Vec::new();
        let mut shown_ranges: Vec<usize> = Vec::new();

        let mut is_first_month = true;

        while current_date <= end_date {
            let layout = WeekLayout::new(current_date);

            // Skip weeks that don't contain filtered months
            if !self.should_render_week(&layout) {
                current_date = current_date
                    .checked_add_signed(chrono::Duration::days(DAYS_IN_WEEK as i64))
                    .unwrap();
                continue;
            }

            let next_week_date = current_date
                .checked_add_signed(chrono::Duration::days(DAYS_IN_WEEK as i64))
                .unwrap();
            let next_layout = WeekLayout::new(next_week_date);

            if let Some((_, month)) = layout.month_start_idx {
                current_month = Some(month);
                if is_first_month {
                    output.push_str(&self.month_border_to_string(&layout, current_month));
                    is_first_month = false;
                }
            }

            self.collect_details(&layout, &mut details_queue);

            output.push_str(&self.week_row_to_string(week_num, &layout, current_month));

            output.push_str(&self.annotations_to_string(
                &layout,
                &mut details_queue,
                &mut shown_ranges,
            ));

            output.push('\n');

            let is_last_week =
                next_week_date.year() > self.calendar.year || next_week_date > end_date;

            if is_last_week {
                let mut month_boundary_idx = None;
                for (idx, &date) in layout.dates.iter().enumerate() {
                    if idx > 0 {
                        let prev_date = layout.dates[idx - 1];
                        if date.month() != prev_date.month() || date.year() != prev_date.year() {
                            month_boundary_idx = Some(idx);
                            break;
                        }
                    }
                }

                if let Some(boundary_idx) = month_boundary_idx {
                    let dashes_before = (boundary_idx - 1) * 5 + 4;
                    let dashes_after = (DAYS_IN_WEEK - boundary_idx) * 5 - 1;
                    output.push_str(&format!(
                        "└{:─<13}┴{:─<before$}┴{:─<after$}┘\n",
                        "",
                        "",
                        "",
                        before = dashes_before,
                        after = dashes_after
                    ));
                } else {
                    output.push_str(&format!(
                        "└{:─<13}┴{:─<width$}┘\n",
                        "",
                        "",
                        width = CALENDAR_WIDTH
                    ));
                }
            } else if let Some((idx, _)) = layout.month_start_idx {
                if idx > 0 {
                    output.push_str(&self.separator_to_string(&layout, current_month));
                }
            } else if next_layout.month_start_idx.is_some()
                && next_week_date <= end_date
                && next_week_date.year() == self.calendar.year
            {
                output.push_str(&self.separator_before_month_to_string(
                    &layout,
                    current_month,
                    &next_layout,
                ));
            }

            current_date = next_week_date;
            week_num += 1;

            if current_date.year() > self.calendar.year {
                break;
            }
        }

        output
    }

    fn month_border_to_string(&self, layout: &WeekLayout, _current_month: Option<u32>) -> String {
        let mut output = String::new();
        if let Some((idx, _)) = layout.month_start_idx {
            if idx > 0 {
                output.push_str("│             ┌");
                let dashes_before = (idx - 1) * 5 + 4;
                for _ in 0..dashes_before {
                    output.push('─');
                }
                output.push('┬');
                let dashes_after = (DAYS_IN_WEEK - idx) * 5 - 1;
                output.push_str(&format!("{:─<width$}┤\n", "", width = dashes_after));
            }
        }
        output
    }

    fn week_row_to_string(
        &self,
        week_num: i32,
        layout: &WeekLayout,
        _current_month: Option<u32>,
    ) -> String {
        let mut output = String::new();
        let month_name = if let Some((_, month)) = layout.month_start_idx {
            MonthInfo::from_month(month).name
        } else {
            ""
        };

        if !month_name.is_empty() {
            output.push_str(&format!("│W{:02} {:<9}", week_num, month_name));
        } else {
            output.push_str(&format!("│W{:02}          ", week_num));
        }

        output.push('│');

        for (idx, &date) in layout.dates.iter().enumerate() {
            let is_month_boundary = if idx > 0 {
                let prev_date = layout.dates[idx - 1];
                date.month() != prev_date.month() || date.year() != prev_date.year()
            } else {
                false
            };

            if is_month_boundary {
                output.push('│');
            }

            output.push_str(&format!(" {:02}", date.day()));

            if idx < 6 {
                let next_date = layout.dates[idx + 1];
                let next_is_boundary =
                    date.month() != next_date.month() || date.year() != next_date.year();
                if next_is_boundary {
                    output.push(' ');
                } else {
                    output.push_str("  ");
                }
            } else {
                output.push(' ');
            }
        }

        output.push('│');
        output
    }

    fn annotations_to_string(
        &self,
        layout: &WeekLayout,
        details_queue: &mut Vec<(NaiveDate, DateDetail)>,
        shown_ranges: &mut Vec<usize>,
    ) -> String {
        let mut output = String::new();
        let week_start = layout.dates[0];
        let week_end = layout.dates[DAYS_IN_WEEK - 1];
        let mut annotations = Vec::new();

        // Collect all details that occur in this week
        let mut details_to_remove = Vec::new();
        for (i, (detail_date, detail)) in details_queue.iter().enumerate() {
            if *detail_date >= week_start && *detail_date <= week_end {
                annotations.push(format!(
                    "{} - {}",
                    detail_date.format("%m/%d"),
                    detail.description
                ));
                details_to_remove.push(i);
            }
        }
        // Remove details in reverse order to maintain indices
        for &i in details_to_remove.iter().rev() {
            details_queue.remove(i);
        }

        // Collect all ranges that overlap with this week
        for (idx, range) in self.calendar.ranges.iter().enumerate() {
            if !shown_ranges.contains(&idx) && range.start <= week_end && range.end >= week_start {
                if let Some(desc) = &range.description {
                    annotations.push(format!(
                        "{} to {} - {}",
                        range.start.format("%m/%d"),
                        range.end.format("%m/%d"),
                        desc
                    ));
                } else {
                    annotations.push(format!(
                        "{} to {}",
                        range.start.format("%m/%d"),
                        range.end.format("%m/%d")
                    ));
                }
                shown_ranges.push(idx);
            }
        }

        // Join all annotations with commas
        output.push_str(&annotations.join(", "));

        output
    }

    fn separator_to_string(&self, layout: &WeekLayout, current_month: Option<u32>) -> String {
        let mut output = String::new();
        output.push_str("│             ├");

        let mut first_bar_idx = None;
        for (idx, &date) in layout.dates.iter().enumerate() {
            let in_month = date.year() == self.calendar.year && Some(date.month()) == current_month;
            let prev_in_month = if idx > 0 {
                let prev_date = layout.dates[idx - 1];
                prev_date.year() == self.calendar.year && Some(prev_date.month()) == current_month
            } else {
                false
            };

            if in_month && !prev_in_month {
                first_bar_idx = Some(idx);
            }
        }

        if let Some(bar_idx) = first_bar_idx {
            if bar_idx > 0 {
                let dashes = (bar_idx - 1) * 5 + 4;
                output.push_str(&format!("{:─<width$}┘", "", width = dashes));
                let spaces = (DAYS_IN_WEEK - bar_idx) * 5 - 1;
                output.push_str(&format!("{: <width$}│\n", "", width = spaces));
            } else {
                output.push_str("───────────────────────────────┤│\n");
            }
        } else {
            output.push_str("───────────────────────────────┤│\n");
        }

        output
    }

    fn separator_before_month_to_string(
        &self,
        _current_layout: &WeekLayout,
        _current_month: Option<u32>,
        next_layout: &WeekLayout,
    ) -> String {
        let mut output = String::new();
        if let Some((next_month_start_idx, _)) = next_layout.month_start_idx {
            if next_month_start_idx == 0 {
                output.push_str("│             ├");
                output.push_str(&format!("{:─<width$}┤", "", width = CALENDAR_WIDTH));
            } else {
                output.push_str("│             │");
                let spaces_before = (next_month_start_idx - 1) * 5 + 4;
                output.push_str(&format!("{: <width$}┌", "", width = spaces_before));
                let dashes = (DAYS_IN_WEEK - 1 - next_month_start_idx) * 5 + 4;
                output.push_str(&format!("{:─<width$}┤", "", width = dashes));
            }
        } else {
            output.push_str("│             │");
            output.push_str(&format!("{: <width$}", "", width = DAYS_IN_WEEK * 4 + 3));
        }

        output.push('\n');
        output
    }

    fn print_header(&self) {
        println!("┌{:─<width$}┐", "", width = HEADER_WIDTH);

        // Center the title
        let title = format!("COMPACT CALENDAR {}", self.calendar.year);
        println!("│{:^width$}│", title, width = HEADER_WIDTH);

        println!("├{:─<width$}┤", "", width = HEADER_WIDTH);
        print!("│              ");
        match self.calendar.week_start {
            WeekStart::Monday => println!("Mon  Tue  Wed  Thu  Fri  Sat  Sun │"),
            WeekStart::Sunday => println!("Sun  Mon  Tue  Wed  Thu  Fri  Sat │"),
        }
    }

    fn print_weeks(&self) {
        let (start_date, end_date) = self.get_filtered_date_range();

        let mut current_date = self.align_to_week_start(start_date);
        let mut week_num = 1;
        let mut current_month: Option<u32> = None;

        let mut details_queue: Vec<(NaiveDate, DateDetail)> = Vec::new();
        let mut shown_ranges: Vec<usize> = Vec::new();

        let mut is_first_month = true;

        while current_date <= end_date {
            let layout = WeekLayout::new(current_date);

            // Skip weeks that don't contain filtered months
            if !self.should_render_week(&layout) {
                current_date = current_date
                    .checked_add_signed(chrono::Duration::days(DAYS_IN_WEEK as i64))
                    .unwrap();
                continue;
            }

            let next_week_date = current_date
                .checked_add_signed(chrono::Duration::days(DAYS_IN_WEEK as i64))
                .unwrap();
            let next_layout = WeekLayout::new(next_week_date);

            if let Some((_, month)) = layout.month_start_idx {
                current_month = Some(month);
                if is_first_month {
                    self.print_month_border(&layout, current_month);
                    is_first_month = false;
                }
            }

            self.collect_details(&layout, &mut details_queue);

            self.print_week_row(week_num, &layout, current_month);

            self.print_annotations(&layout, &mut details_queue, &mut shown_ranges);

            println!();

            let is_last_week =
                next_week_date.year() > self.calendar.year || next_week_date > end_date;

            if is_last_week {
                let mut month_boundary_idx = None;
                for (idx, &date) in layout.dates.iter().enumerate() {
                    if idx > 0 {
                        let prev_date = layout.dates[idx - 1];
                        if date.month() != prev_date.month() || date.year() != prev_date.year() {
                            month_boundary_idx = Some(idx);
                            break;
                        }
                    }
                }

                if let Some(boundary_idx) = month_boundary_idx {
                    let dashes_before = (boundary_idx - 1) * 5 + 4;
                    let dashes_after = (DAYS_IN_WEEK - boundary_idx) * 5 - 1;
                    println!(
                        "└{:─<13}┴{:─<before$}┴{:─<after$}┘",
                        "",
                        "",
                        "",
                        before = dashes_before,
                        after = dashes_after
                    );
                } else {
                    println!("└{:─<13}┴{:─<width$}┘", "", "", width = CALENDAR_WIDTH);
                }
            } else if let Some((idx, _)) = layout.month_start_idx {
                if idx > 0 {
                    self.print_separator(&layout, current_month);
                }
            } else if next_layout.month_start_idx.is_some()
                && next_week_date <= end_date
                && next_week_date.year() == self.calendar.year
            {
                self.print_separator_before_month(&layout, current_month, &next_layout);
            }

            current_date = next_week_date;
            week_num += 1;

            if current_date.year() > self.calendar.year {
                break;
            }
        }
    }

    fn align_to_week_start(&self, date: NaiveDate) -> NaiveDate {
        let mut aligned = date;
        while self.calendar.get_weekday_num(aligned) != 0 {
            aligned = aligned.pred_opt().unwrap();
        }
        aligned
    }

    fn get_date_color(&self, date: NaiveDate) -> Option<String> {
        // In work mode, never color weekends
        if self.calendar.color_mode == ColorMode::Work
            && (date.weekday() == Weekday::Sat || date.weekday() == Weekday::Sun)
        {
            return None;
        }

        // Check if date has a specific color
        if let Some(detail) = self.calendar.details.get(&date) {
            if let Some(color) = &detail.color {
                return Some(color.clone());
            }
        }

        // Check if date is in a range
        for range in &self.calendar.ranges {
            if date >= range.start && date <= range.end {
                return Some(range.color.clone());
            }
        }

        None
    }

    fn print_month_border(&self, layout: &WeekLayout, _current_month: Option<u32>) {
        if let Some((idx, _)) = layout.month_start_idx {
            if idx > 0 {
                print!("│             ┌");
                let dashes_before = (idx - 1) * 5 + 4;
                for _ in 0..dashes_before {
                    print!("─");
                }
                print!("┬");
                let dashes_after = (DAYS_IN_WEEK - idx) * 5 - 1;
                println!("{:─<width$}┤", "", width = dashes_after);
            }
        }
    }

    fn collect_details(
        &self,
        layout: &WeekLayout,
        details_queue: &mut Vec<(NaiveDate, DateDetail)>,
    ) {
        for &date in &layout.dates {
            if let Some(detail) = self.calendar.details.get(&date) {
                if !details_queue.iter().any(|(d, _)| d == &date) {
                    details_queue.push((date, detail.clone()));
                }
            }
        }
    }

    fn print_week_row(&self, week_num: i32, layout: &WeekLayout, _current_month: Option<u32>) {
        let month_name = if let Some((_, month)) = layout.month_start_idx {
            MonthInfo::from_month(month).name
        } else {
            ""
        };

        if !month_name.is_empty() {
            print!("│W{:02} {:<9}", week_num, month_name);
        } else {
            print!("│W{:02}          ", week_num);
        }

        print!("│");

        for (idx, &date) in layout.dates.iter().enumerate() {
            let is_month_boundary = if idx > 0 {
                let prev_date = layout.dates[idx - 1];
                date.month() != prev_date.month() || date.year() != prev_date.year()
            } else {
                false
            };

            if is_month_boundary {
                print!("│");
            }

            let today = chrono::Local::now().date_naive();
            let is_today = date == today;
            let is_past =
                self.calendar.past_date_display == PastDateDisplay::Strikethrough && date < today;

            let is_weekend = self.calendar.weekend_display == WeekendDisplay::Dimmed
                && (date.weekday() == Weekday::Sat || date.weekday() == Weekday::Sun);

            if let Some(color) = self.get_date_color(date) {
                let mut style = if is_weekend {
                    ColorCodes::get_dimmed_bg_color(&color)
                } else {
                    ColorCodes::get_bg_color(&color)
                };

                if ColorCodes::is_color_disabled() {
                    print!(" {:02}", date.day());
                } else {
                    style = style.fg_color(ColorCodes::black_text().get_fg_color());

                    let mut effects = Effects::new();
                    if is_past {
                        effects |= ColorCodes::strikethrough();
                    }
                    if is_today {
                        effects |= ColorCodes::underline();
                    }
                    style = style.effects(effects);

                    print!(
                        " {}{:02}{}",
                        style.render(),
                        date.day(),
                        style.render_reset()
                    );
                }
            } else if ColorCodes::is_color_disabled() {
                print!(" {:02}", date.day());
            } else {
                let mut style = Style::new();
                let mut effects = Effects::new();

                if is_past {
                    effects |= ColorCodes::strikethrough();
                }
                if is_today {
                    effects |= ColorCodes::underline();
                }
                if is_weekend {
                    effects |= ColorCodes::dim();
                }

                style = style.effects(effects);

                if effects == Effects::new() {
                    print!(" {:02}", date.day());
                } else {
                    print!(
                        " {}{:02}{}",
                        style.render(),
                        date.day(),
                        style.render_reset()
                    );
                }
            }

            if idx < 6 {
                let next_date = layout.dates[idx + 1];
                let next_is_boundary =
                    date.month() != next_date.month() || date.year() != next_date.year();
                if next_is_boundary {
                    print!(" ");
                } else {
                    print!("  ");
                }
            } else {
                print!(" ");
            }
        }

        print!("│");
    }

    fn print_annotations(
        &self,
        layout: &WeekLayout,
        details_queue: &mut Vec<(NaiveDate, DateDetail)>,
        shown_ranges: &mut Vec<usize>,
    ) {
        let week_start = layout.dates[0];
        let week_end = layout.dates[DAYS_IN_WEEK - 1];
        let mut first = true;

        // Collect and print all details that occur in this week
        let mut details_to_remove = Vec::new();
        for (i, (detail_date, detail)) in details_queue.iter().enumerate() {
            if *detail_date >= week_start && *detail_date <= week_end {
                if !first {
                    print!(", ");
                }
                first = false;

                if ColorCodes::is_color_disabled() {
                    print!("{} - {}", detail_date.format("%m/%d"), detail.description);
                } else if let Some(color) = &detail.color {
                    let style = ColorCodes::get_bg_color(color)
                        .fg_color(ColorCodes::black_text().get_fg_color());
                    print!(
                        "{}{} - {}{}",
                        style.render(),
                        detail_date.format("%m/%d"),
                        detail.description,
                        style.render_reset()
                    );
                } else {
                    print!("{} - {}", detail_date.format("%m/%d"), detail.description);
                }
                details_to_remove.push(i);
            }
        }
        // Remove details in reverse order to maintain indices
        for &i in details_to_remove.iter().rev() {
            details_queue.remove(i);
        }

        // Collect and print all ranges that overlap with this week
        for (idx, range) in self.calendar.ranges.iter().enumerate() {
            if !shown_ranges.contains(&idx) && range.start <= week_end && range.end >= week_start {
                if !first {
                    print!(", ");
                }
                first = false;

                if ColorCodes::is_color_disabled() {
                    if let Some(desc) = &range.description {
                        print!(
                            "{} to {} - {}",
                            range.start.format("%m/%d"),
                            range.end.format("%m/%d"),
                            desc
                        );
                    } else {
                        print!(
                            "{} to {}",
                            range.start.format("%m/%d"),
                            range.end.format("%m/%d")
                        );
                    }
                } else {
                    let style = ColorCodes::get_bg_color(&range.color)
                        .fg_color(ColorCodes::black_text().get_fg_color());

                    if let Some(desc) = &range.description {
                        print!(
                            "{}{} to {} - {}{}",
                            style.render(),
                            range.start.format("%m/%d"),
                            range.end.format("%m/%d"),
                            desc,
                            style.render_reset()
                        );
                    } else {
                        print!(
                            "{}{} to {}{}",
                            style.render(),
                            range.start.format("%m/%d"),
                            range.end.format("%m/%d"),
                            style.render_reset()
                        );
                    }
                }
                shown_ranges.push(idx);
            }
        }
    }

    fn print_separator(&self, layout: &WeekLayout, current_month: Option<u32>) {
        print!("│             ├");
        let mut first_bar_idx = None;
        for (idx, &date) in layout.dates.iter().enumerate() {
            let in_month = date.year() == self.calendar.year && Some(date.month()) == current_month;
            let prev_in_month = if idx > 0 {
                let prev_date = layout.dates[idx - 1];
                prev_date.year() == self.calendar.year && Some(prev_date.month()) == current_month
            } else {
                false
            };

            if in_month && !prev_in_month {
                first_bar_idx = Some(idx);
            }
        }

        if let Some(bar_idx) = first_bar_idx {
            if bar_idx > 0 {
                let dashes = (bar_idx - 1) * 5 + 4;
                print!("{:─<width$}┘", "", width = dashes);
                let spaces = (DAYS_IN_WEEK - bar_idx) * 5 - 1;
                println!("{: <width$}│", "", width = spaces);
            } else {
                println!("{:─<31}┤│", "");
            }
        } else {
            println!("{:─<31}┤│", "");
        }
    }

    fn print_separator_before_month(
        &self,
        _current_layout: &WeekLayout,
        _current_month: Option<u32>,
        next_layout: &WeekLayout,
    ) {
        if let Some((next_month_start_idx, _)) = next_layout.month_start_idx {
            if next_month_start_idx == 0 {
                print!("│             ├");
                print!("{:─<width$}┤", "", width = CALENDAR_WIDTH);
            } else {
                print!("│             │");
                let spaces_before = (next_month_start_idx - 1) * 5 + 4;
                print!("{: <width$}┌", "", width = spaces_before);
                let dashes = (DAYS_IN_WEEK - 1 - next_month_start_idx) * 5 + 4;
                print!("{:─<width$}┤", "", width = dashes);
            }
        } else {
            print!("│             │");
            print!("{: <width$}", "", width = DAYS_IN_WEEK * 4 + 3);
        }

        println!();
    }
}
