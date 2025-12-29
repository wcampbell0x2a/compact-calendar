use compact_calendar::models::{ColorMode, PastDateDisplay, WeekStart, WeekendDisplay};
use compact_calendar::rendering::CalendarRenderer;
use std::path::PathBuf;

fn create_calendar_from_config(year: i32, config_path: &str) -> String {
    let config = compact_calendar::load_config(&PathBuf::from(config_path));
    let calendar = compact_calendar::build_calendar(
        year,
        WeekStart::Monday,
        WeekendDisplay::Normal,
        ColorMode::Normal,
        PastDateDisplay::Normal,
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
    let config = compact_calendar::load_config(&PathBuf::from("tests/fixtures/simple.toml"));
    let calendar = compact_calendar::build_calendar(
        2024,
        WeekStart::Sunday,
        WeekendDisplay::Normal,
        ColorMode::Normal,
        PastDateDisplay::Normal,
        config,
    );

    let renderer = CalendarRenderer::new(&calendar);
    let output = renderer.render_to_string();
    insta::assert_snapshot!(output);
}
