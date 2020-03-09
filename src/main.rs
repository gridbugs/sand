use simon::Arg;
use std::str::FromStr;
use std::time::Duration;

struct ParsableDuration(Duration);

impl FromStr for ParsableDuration {
    type Err = parse_duration::parse::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_duration::parse(s).map(ParsableDuration)
    }
}

struct Args {
    duration: Duration,
    interval: Duration,
}

impl Args {
    fn arg() -> impl Arg<Item = Self> {
        simon::args_map! {
            let {
                duration = simon::free::<ParsableDuration>().vec_singleton();
                interval = simon::opt::<ParsableDuration>("i", "interval", "interval to update display (default 1s)", "DURATION")
                    .with_default(ParsableDuration(Duration::from_secs(1)));
            } in {
                Self {
                    duration: duration.0,
                    interval: interval.0,
                }
            }
        }
    }
}

fn print_remaining(remaining: Duration) {
    use std::io::Write;
    let mut stdout = std::io::stdout();
    let remaining = chrono::Duration::from_std(remaining).unwrap();
    let weeks = remaining.num_weeks();
    let days = remaining.num_days() % 7;
    let hours = remaining.num_hours() % 24;
    let minutes = remaining.num_minutes() % 60;
    let seconds = remaining.num_seconds() % 60;
    let millis = remaining.num_milliseconds() % 1000;
    let mut started = false;
    write!(stdout, "\r").unwrap();
    if weeks > 0 || started {
        write!(stdout, "{}w ", weeks).unwrap();
        started = true;
    }
    if days > 0 || started {
        write!(stdout, "{}d ", days).unwrap();
        started = true;
    }
    if hours > 0 || started {
        write!(stdout, "{}h ", hours).unwrap();
        started = true;
    }
    if minutes > 0 || started {
        write!(stdout, "{}m ", minutes).unwrap();
    }
    write!(stdout, "{}", seconds).unwrap();
    if millis > 0 {
        write!(stdout, ".").unwrap();
        let h = millis / 100;
        let t = (millis / 10) % 10;
        let o = millis % 10;
        if h > 0 || t > 0 || o > 0 {
            write!(stdout, "{}", h).unwrap();
        }
        if t > 0 || o > 0 {
            write!(stdout, "{}", t).unwrap();
        }
        if o > 0 {
            write!(stdout, "{}", o).unwrap();
        }
    }
    write!(stdout, "s").unwrap();
    stdout.flush().unwrap();
}

async fn print_intervals(total_duration: Duration, interval_duration: Duration) {
    let mut remaining = total_duration;
    print_remaining(remaining);
    let mut interval = tokio::time::interval(interval_duration);
    loop {
        interval.tick().await;
        print_remaining(remaining);
        remaining = if let Some(remaining) = remaining.checked_sub(interval_duration) {
            remaining
        } else {
            Duration::from_millis(0)
        };
    }
}

#[tokio::main]
async fn main() {
    let Args { duration, interval } = Args::arg().with_help_default().parse_env_or_exit();
    tokio::select! {
        _ = print_intervals(duration, interval) => {}
        _ = tokio::time::delay_for(duration) => {}
    }
    print_remaining(Duration::from_secs(0));
    println!("");
}
