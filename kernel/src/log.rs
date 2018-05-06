macro_rules! error {
    ($thing:expr, $($extra:tt)*) => {
        {
            use terminal::TerminalOutput;
            ::terminal::STDOUT.write().write_string_colored("[error] ", color!(Red on Black))
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
            ::terminal::STDOUT.write().write_string_colored("[warn]  ", color!(LightRed on Black))
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
            ::terminal::STDOUT.write().write_string_colored("[info]  ", color!(LightBlue on Black))
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
            ::terminal::STDOUT.write().write_string_colored("[debug] ", color!(Cyan on Black))
                .expect("Error logging");
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
            use terminal::TerminalOutput;
            ::terminal::STDOUT.write().write_string_colored("[trace] ", color!(White on Black))
                .expect("Error logging");
            println!($thing, $($extra)*);
        }
    };

    ($thing:expr) => {
        trace!($thing,)
    }
}
