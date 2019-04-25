use crate::log_facade::{self, Log, Record, Level, Metadata};

static LOGGER: Logger = Logger;

macro_rules! error {
    ($thing:expr, $($extra:tt)*) => {
        {
            use crate::terminal::TerminalOutput;
            crate::terminal::STDOUT.write().write_string_colored("[error] ", color!(Red on Black))
                .expect("Error logging");
            serial_print!("[error] " );
            serial_println!($thing, $($extra)*);
            println!($thing, $($extra)*);
        }
    };

    ($thing:expr) => {
        error!($thing,)
    }
}


macro_rules! warn {
    ($thing:expr, $($extra:tt)*) => {
        {
            use crate::terminal::TerminalOutput;
            crate::terminal::STDOUT.write().write_string_colored("[warn]  ", color!(LightRed on Black))
                .expect("Error logging");
            serial_print!("[warn]  " );
            serial_println!($thing, $($extra)*);
            println!($thing, $($extra)*);
        }
    };

    ($thing:expr) => {
        warn!($thing,)
    }
}

macro_rules! info {
    ($thing:expr, $($extra:tt)*) => {
        {
            use crate::terminal::TerminalOutput;
            crate::terminal::STDOUT.write().write_string_colored("[info]  ", color!(LightBlue on Black))
                .expect("Error logging");
            serial_print!("[info]  " );
            serial_println!($thing, $($extra)*);
            println!($thing, $($extra)*);
        }
    };

    ($thing:expr) => {
        info!($thing,)
    }
}

macro_rules! debug {
    ($thing:expr, $($extra:tt)*) => {
        #[cfg(feature = "debug")]
        {
            use crate::terminal::TerminalOutput;
            crate::terminal::STDOUT.write().write_string_colored("[debug] ", color!(Cyan on Black))
                .expect("Error logging");
            serial_print!("[debug] " );
            serial_println!($thing, $($extra)*);
            println!($thing, $($extra)*);
        }
    };

    ($thing:expr) => {
        debug!($thing,)
    }
}

macro_rules! trace {
    ($thing:expr, $($extra:tt)*) => {
        #[cfg(feature = "trace")]
        {
            use crate::terminal::TerminalOutput;
            crate::terminal::STDOUT.write().write_string_colored("[trace] ", color!(White on Black))
                .expect("Error logging");
            serial_print!("[trace] " );
            serial_println!($thing, $($extra)*);
            println!($thing, $($extra)*);
        }
    };

    ($thing:expr) => {
        trace!($thing,)
    }
}

struct Logger;

// `return` statements and `#[allow]` required here because of the `cfg`s and how log levels work
#[allow(unreachable_code)]
const fn log_level() -> Level {
    #[cfg(feature = "trace")]
    return Level::Trace;

    #[cfg(feature = "debug")]
    return Level::Debug;

    Level::Info
}

impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= log_level()
    }

    fn log(&self, record: &Record) {
        use crate::drivers::serial;
        use crate::terminal::{STDOUT, TerminalOutput};
        use core::fmt::Write;

        if self.enabled(record.metadata()) {
            let (label, color) = match record.level() {
                Level::Trace => ("[trace] ", color!(White on Black)),
                Level::Debug => ("[debug] ", color!(Cyan on Black)),
                Level::Info  => ("[info]  ", color!(LightBlue on Black)),
                Level::Warn  => ("[warn]  ", color!(LightRed on Black)),
                Level::Error => ("[error] ", color!(Red on Black)),
            };

            STDOUT.write().write_string_colored(label, color).expect("Error logging");

            let message = format!("{}: {}\n", record.target(), record.args());

            STDOUT.write().write_string(&message)
                .expect("Error logging");

            write!(serial::PORT_1.lock(), "{}", label).unwrap();
            write!(serial::PORT_1.lock(), "{}", message).unwrap();
        }
    }

    fn flush(&self) {}
}

pub fn init() {
    log_facade::set_logger(&LOGGER)
        .map(|()| log_facade::set_max_level(log_level().to_level_filter()))
        .expect("Error setting logger!");
}