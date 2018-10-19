extern crate libc;
extern crate getopts;

use getopts::Options;

#[derive(Debug)]
struct BeepOptions {
    freq: u32,
    duration: u32,
    verbose: bool,
}

const KIOCSOUND: u64 = 0x4B2F; //start sound gen (0 -> off)

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let options = match parse_args(args) {
        Some(o) => o,
        None => return,
    };

    if !beep_so(options.freq, options.duration) {
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
        let f = if freq == 0 { 0 } else { 1193180 / freq };

        unsafe {
            // TODO can fail
            libc::ioctl(libc::STDOUT_FILENO, KIOCSOUND, f) == 0
        }
    }
}

fn beep_so(freq: u32, duration: u32) -> bool {
    let beeper = BeepSo {};
    if !beeper.beep(freq) {
        println!("test_beep_so: beep returned false");
        return false;
    }
    std::thread::sleep(std::time::Duration::from_millis(duration as u64));
    beeper.beep(0);
    true
}

fn parse_args(args: Vec<String>) -> Option<BeepOptions> {
    let program = args[0].clone();
    let mut options = Options::new();
    options
        .optflag("h", "help", "print this help")
        .optflag("v", "verbose", "print more info")
        .optopt("f", "frequency", "sets desired frequency (min max)", "FREQ")
        .optopt(
            "d",
            "duration",
            "sets duration of a beep in ms",
            "MILLISECONDS",
        );
    let matches = match options.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => panic!(f.to_string()),
    };
    if matches.opt_present("h") {
        print_usage(&program, options);
        return None;
    }
    let verbose = matches.opt_present("v");
    let freq = match matches.opt_get::<u32>("f") {
        Ok(f) => f.unwrap_or(440),
        Err(e) => {
            print_error(&e.to_string(), "frequency", options);
            return None;
            // panic!(e.to_string())
        }
    };

    let duration = match matches.opt_get::<u32>("d") {
        Ok(d) => d.unwrap_or(1000),
        Err(e) => {
            print_error(&e.to_string(), "duration", options);
            return None;
            // panic!(e.to_string())
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

fn print_error(err: &str, option: &str, opts: Options) {
    let msg = format!("ERROR parsing option {} \n {}", option, err);
    print!("{}", opts.usage(&msg));
}
