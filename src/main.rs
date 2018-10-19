// basic alternative to https://github.com/johnath/beep

extern crate getopts;
extern crate libc;

use getopts::Options;

#[derive(Debug)]
struct BeepOptions {
    freq: u32,
    duration: u32,
    verbose: bool,
}

const CLOCK_TICK_RATE: u32 = 1193180;
const DEFAULT_FREQ: u32 = 440;
const DEFAULT_DURATION: u32 = 1000;
const KIOCSOUND: u64 = 0x4B2F; //start sound gen (0 -> off)

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let options = match parse_args(args) {
        Some(o) => o,
        None => return, // something bad has happend, or --help has been requested
    };

    // TODO cap frequency and duration
    if !beep_so(options.freq, options.duration) {
        // ioctl may need root
        if options.verbose {
            println!("failed. fallback to 'BELL'");
        }
        unsafe { libc::putchar(0x7) };
    }
}

pub trait Beep {
    // TODO return Result
    fn beep(&self, freq: u32) -> bool;
}

pub struct BeepSo {}
impl Beep for BeepSo {
    fn beep(&self, freq: u32) -> bool {
        let f = if freq == 0 { 0 } else { CLOCK_TICK_RATE / freq };

        unsafe {
            // TODO can fail
            libc::ioctl(libc::STDOUT_FILENO, KIOCSOUND, f) == 0
        }
    }
}

fn beep_so(freq: u32, duration: u32) -> bool {
    let beeper = BeepSo {};
    if !beeper.beep(freq) {
        return false;
    }
    std::thread::sleep(std::time::Duration::from_millis(duration as u64));
    beeper.beep(0);
    true
}

// TODO return Result so print_error can be moved to main
fn parse_args(args: Vec<String>) -> Option<BeepOptions> {
    let program = args[0].clone();
    let mut options = Options::new();
    options
        .optflag("h", "help", "print this message")
        .optflag("v", "verbose", "print more info")
        .optopt(
            "f",
            "frequency",
            "sets desired frequency (default 440)",
            "FREQ",
        ).optopt(
            "d",
            "duration",
            "sets duration of a beep in ms (default 1000)",
            "MILLIS",
        );
    let matches = match options.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => {
            print_error(&e.to_string(), "parsing arguments failed", options);
            return None;
        }
    };
    if matches.opt_present("h") {
        print_usage(&program, options);
        return None;
    }
    let verbose = matches.opt_present("v");
    let freq = match matches.opt_get::<u32>("f") {
        Ok(f) => f.unwrap_or(DEFAULT_FREQ),
        Err(e) => {
            print_error(&e.to_string(), "parsing frequency failed", options);
            return None;
        }
    };

    let duration = match matches.opt_get::<u32>("d") {
        Ok(d) => d.unwrap_or(DEFAULT_DURATION),
        Err(e) => {
            print_error(&e.to_string(), "parsing duration failed", options);
            return None;
        }
    };

    Some(BeepOptions {
        freq,
        duration,
        verbose,
    })
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("Usage: {} [options]", program);
    print!("{}", opts.usage(&brief));
}

fn print_error(err: &str, msg: &str, opts: Options) {
    let msg = format!("ERROR {} \n {}", msg, err);
    print!("{}", opts.usage(&msg));
}
