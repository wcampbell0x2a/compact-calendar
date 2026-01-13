use compact_calendar_cli::models::{
    ColorMode, MonthFilter, PastDateDisplay, WeekStart, WeekendDisplay,
};
use compact_calendar_cli::rendering::CalendarRenderer;
use std::path::PathBuf;

fn create_calendar_from_config(year: i32, config_path: &str) -> String {
    create_calendar_from_config_with_filter(year, config_path, MonthFilter::All)
}

fn create_calendar_from_config_with_filter(
    year: i32,
    config_path: &str,
    month_filter: MonthFilter,
) -> String {
    let config = compact_calendar_cli::load_config(&PathBuf::from(config_path));
    let calendar = compact_calendar_cli::build_calendar(
        year,
        WeekStart::Monday,
        WeekendDisplay::Normal,
        ColorMode::Normal,
        PastDateDisplay::Normal,
        month_filter,
        config,
    );

    let renderer = CalendarRenderer::new(&calendar);
    renderer.render_to_string()
}

#[test]
fn test_simple_2024() {
    let output = create_calendar_from_config(2024, "tests/fixtures/simple.toml");
    insta::assert_snapshot!(output);
}

#[test]
fn test_simple_2025() {
    let output = create_calendar_from_config(2025, "tests/fixtures/simple.toml");
    insta::assert_snapshot!(output);
}

#[test]
fn test_quarters_2023() {
    let output = create_calendar_from_config(2023, "tests/fixtures/quarters.toml");
    insta::assert_snapshot!(output);
}

#[test]
fn test_quarters_2024() {
    let output = create_calendar_from_config(2024, "tests/fixtures/quarters.toml");
    insta::assert_snapshot!(output);
}

#[test]
fn test_empty_2024() {
    let output = create_calendar_from_config(2024, "tests/fixtures/empty.toml");
    insta::assert_snapshot!(output);
}

#[test]
fn test_empty_2025() {
    let output = create_calendar_from_config(2025, "tests/fixtures/empty.toml");
    insta::assert_snapshot!(output);
}

#[test]
fn test_sunday_start_2024() {
    let config = compact_calendar_cli::load_config(&PathBuf::from("tests/fixtures/simple.toml"));
    let calendar = compact_calendar_cli::build_calendar(
        2024,
        WeekStart::Sunday,
        WeekendDisplay::Normal,
        ColorMode::Normal,
        PastDateDisplay::Normal,
        MonthFilter::All,
        config,
    );

    let renderer = CalendarRenderer::new(&calendar);
    let output = renderer.render_to_string();
    insta::assert_snapshot!(output);
}

// Month filtering tests

#[test]
fn test_single_month_by_number_march_2026() {
    let output = create_calendar_from_config_with_filter(
        2026,
        "tests/fixtures/empty.toml",
        MonthFilter::Single(3),
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_single_month_january_2026() {
    let output = create_calendar_from_config_with_filter(
        2026,
        "tests/fixtures/empty.toml",
        MonthFilter::Single(1),
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_single_month_december_2026() {
    let output = create_calendar_from_config_with_filter(
        2026,
        "tests/fixtures/empty.toml",
        MonthFilter::Single(12),
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_single_month_february_2024_leap_year() {
    let output = create_calendar_from_config_with_filter(
        2024,
        "tests/fixtures/empty.toml",
        MonthFilter::Single(2),
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_current_with_following_3_months_2026() {
    // This simulates January + 3 following months = Jan, Feb, Mar, Apr
    let output = create_calendar_from_config_with_filter(
        2026,
        "tests/fixtures/empty.toml",
        MonthFilter::CurrentWithFollowing(3),
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_single_month_with_events_march_2024() {
    let output = create_calendar_from_config_with_filter(
        2024,
        "tests/fixtures/simple.toml",
        MonthFilter::Single(3),
    );
    insta::assert_snapshot!(output);
}

#[test]
fn test_three_months_with_quarters_2024() {
    // Test Q1: Jan, Feb, Mar
    let output = create_calendar_from_config_with_filter(
        2024,
        "tests/fixtures/quarters.toml",
        MonthFilter::CurrentWithFollowing(2),
    );
    insta::assert_snapshot!(output);
}
