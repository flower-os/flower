macro_rules! error {
    ($thing:expr, $($extra:tt)*) => {
        {
            use terminal::TerminalOutput;
            ::terminal::STDOUT.write().write_string_colored("[Error] ", color!(Red on Black))
                .expect("Error logging");
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
            use terminal::TerminalOutput;
            ::terminal::STDOUT.write().write_string_colored("[Warn] ", color!(LightRed on Black))
                .expect("Error logging");
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
            use terminal::TerminalOutput;
            ::terminal::STDOUT.write().write_string_colored("[Info] ", color!(LightBlue on Black))
                .expect("Error logging");
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
            use terminal::TerminalOutput;
            ::terminal::STDOUT.write().write_string_colored("[Debug] ", color!(Cyan on Black))
                .expect("Error logging");
            println!($thing, $($extra)*);
        }
    };

    ($thing:expr) => {
        debug!($thing,)
    }
}

#[allow(dead_code)] // Not used *yet*
macro_rules! trace {
    ($thing:expr, $($extra:tt)*) => {
        #[cfg(feature = "trace")]
        {
            use terminal::TerminalOutput;
            ::terminal::STDOUT.write().write_string_colored("[Trace] ", color!(White on Black))
                .expect("Error logging");
            println!($thing, $($extra)*);
        }
    };

    ($thing:expr) => {
        trace!($thing,)
    }
}
