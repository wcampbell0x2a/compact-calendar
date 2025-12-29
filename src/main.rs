use chrono::Datelike;
use clap::Parser;
use compact_calendar::models::{ColorMode, PastDateDisplay, WeekStart, WeekendDisplay};
use compact_calendar::rendering::CalendarRenderer;
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct Args {
    /// Year to display (defaults to current year)
    #[arg(short, long)]
    year: Option<i32>,

    /// Path to TOML configuration file with date details
    #[arg(short, long, default_value = "calendar.toml")]
    config: PathBuf,

    /// Week starts on Sunday (default is Monday)
    #[arg(short, long)]
    sunday: bool,

    /// Don't dim weekend dates (by default weekends are dimmed)
    #[arg(long)]
    no_dim_weekends: bool,

    /// Work mode: never apply colors to Saturday/Sunday
    #[arg(short, long)]
    work: bool,

    /// Don't strikethrough past dates (by default past dates are crossed out)
    #[arg(long)]
    no_strikethrough_past: bool,
}

fn main() {
    let args = Args::parse();
    let year = args.year.unwrap_or_else(|| chrono::Local::now().year());

    let config = compact_calendar::load_config(&args.config);

    let week_start = if args.sunday {
        WeekStart::Sunday
    } else {
        WeekStart::Monday
    };

    let weekend_display = if args.no_dim_weekends {
        WeekendDisplay::Normal
    } else {
        WeekendDisplay::Dimmed
    };

    let color_mode = if args.work {
        ColorMode::Work
    } else {
        ColorMode::Normal
    };

    let past_date_display = if args.no_strikethrough_past {
        PastDateDisplay::Normal
    } else {
        PastDateDisplay::Strikethrough
    };

    let calendar = compact_calendar::build_calendar(
        year,
        week_start,
        weekend_display,
        color_mode,
        past_date_display,
        config,
    );

    let renderer = CalendarRenderer::new(&calendar);
    renderer.render();
}
