//! Lang items

#[lang = "eh_personality"]
extern fn eh_personality() {}

// TODO error message print
#[lang = "panic_fmt"]
#[no_mangle]
#[allow(private_no_mangle_fns)] // publicity is not required, but no mangle is
extern fn rust_begin_panic() -> ! {
    loop {
        // Spin
    }
}
