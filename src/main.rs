use chrono::Datelike;
use clap::Parser;
use compact_calendar_cli::models::{
    ColorMode, MonthFilter, PastDateDisplay, WeekStart, WeekendDisplay,
};
use compact_calendar_cli::rendering::CalendarRenderer;
use std::path::PathBuf;

/// Restore the default SIGPIPE signal handler.
///
/// Rust's pre-main initialization code sets SIGPIPE to ignore. This
/// disposition is inherited by child processes through execve(),
/// which it shouldn't be. See signal(7):
///
///   "During an execve(2), the dispositions of handled signals are
///    reset to the default; the dispositions of ignored signals are
///    left unchanged."
fn restore_sigpipe_default() {
    unsafe {
        libc::signal(libc::SIGPIPE, libc::SIG_DFL);
    }
}

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

    /// Display a specific month (number 1-12, name like "march", or "current")
    #[arg(short = 'm', long)]
    month: Option<String>,

    /// Display current month plus N additional months (requires --month current)
    #[arg(short = 'f', long)]
    following_months: Option<u32>,
}

fn main() {
    restore_sigpipe_default();
    let args = Args::parse();
    let year = args.year.unwrap_or_else(|| chrono::Local::now().year());

    let config = compact_calendar_cli::load_config(&args.config);

    let week_start = WeekStart::from_sunday_flag(args.sunday);
    let weekend_display = WeekendDisplay::from_no_dim_flag(args.no_dim_weekends);
    let color_mode = ColorMode::from_work_flag(args.work);
    let past_date_display = PastDateDisplay::from_no_strikethrough_flag(args.no_strikethrough_past);
    let month_filter = MonthFilter::from_cli_args(args.month.as_deref(), args.following_months)
        .unwrap_or_else(|e| {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        });

    let calendar = compact_calendar_cli::build_calendar(
        year,
        week_start,
        weekend_display,
        color_mode,
        past_date_display,
        month_filter,
        config,
    );

    let renderer = CalendarRenderer::new(&calendar);
    renderer.render();
}
