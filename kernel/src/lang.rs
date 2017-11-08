//! Lang items

#[lang = "eh_personality"]
extern fn eh_personality() {}

// TODO error message print
#[lang = "panic_fmt"]
extern fn rust_begin_panic() -> ! {
    loop {
        // Spin
    }
}