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

#[derive(Default)]
struct Printer {
    prev_line: usize,
    buf: String,
    max_decimals: usize,
}
impl Printer {
    fn print_remaining(&mut self, remaining: Duration) {
        use std::fmt::Write;
        let remaining = chrono::Duration::from_std(remaining).unwrap();
        let weeks = remaining.num_weeks();
        let days = remaining.num_days() % 7;
        let hours = remaining.num_hours() % 24;
        let minutes = remaining.num_minutes() % 60;
        let seconds = remaining.num_seconds() % 60;
        let millis = remaining.num_milliseconds() % 1000;
        let mut started = false;
        self.buf.clear();
        if weeks > 0 || started {
            write!(self.buf, "{}w ", weeks).unwrap();
            started = true;
        }
        if days > 0 || started {
            write!(self.buf, "{}d ", days).unwrap();
            started = true;
        }
        if hours > 0 || started {
            write!(self.buf, "{}h ", hours).unwrap();
            started = true;
        }
        if minutes > 0 || started {
            write!(self.buf, "{}m ", minutes).unwrap();
        }
        write!(self.buf, "{}", seconds).unwrap();
        if millis > 0 || self.max_decimals > 0 {
            write!(self.buf, ".").unwrap();
            let h = millis / 100;
            let t = (millis / 10) % 10;
            let o = millis % 10;
            let print_o = o > 0 || self.max_decimals >= 3;
            let print_t = t > 0 || print_o || self.max_decimals >= 2;
            let print_h = h > 0 || print_t || self.max_decimals >= 1;
            if print_h {
                write!(self.buf, "{}", h).unwrap();
                self.max_decimals = self.max_decimals.max(1);
            }
            if print_t {
                write!(self.buf, "{}", t).unwrap();
                self.max_decimals = self.max_decimals.max(2);
            }
            if print_o {
                write!(self.buf, "{}", o).unwrap();
                self.max_decimals = self.max_decimals.max(3);
            }
        }
        write!(self.buf, "s").unwrap();
        print!("\r{}", self.buf);
        let len = self.buf.len();
        if let Some(pad) = self.prev_line.checked_sub(len) {
            for _ in 0..pad {
                print!(" ");
            }
        }
        self.prev_line = len;
        use std::io::Write as IoWrite;
        std::io::stdout().flush().unwrap();
    }
}

async fn print_intervals(total_duration: Duration, interval_duration: Duration) {
    let mut remaining = total_duration;
    let mut printer = Printer::default();
    printer.print_remaining(remaining);
    let mut interval = tokio::time::interval(interval_duration);
    loop {
        interval.tick().await;
        printer.print_remaining(remaining);
        remaining = if let Some(remaining) = remaining.checked_sub(interval_duration) {
            remaining
        } else {
            Duration::from_millis(0)
        };
    }
}

#[tokio::main]
async fn main() {
    print!("{}[8", (27u8 as char));
    let Args { duration, interval } = Args::arg().with_help_default().parse_env_or_exit();
    tokio::select! {
        _ = print_intervals(duration, interval) => {}
        _ = tokio::time::delay_for(duration) => {}
    }
    print!("\r");
}
