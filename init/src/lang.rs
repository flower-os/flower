use core::panic::PanicInfo;

// A note on the `#[no_mangle]`s:
// Apparently, removing them makes it link-error with undefined symbols, so we include them

#[lang = "eh_personality"]
#[no_mangle]
extern fn eh_personality() {}

// TODO
#[panic_handler]
#[no_mangle]
extern fn panic_fmt(_info: &PanicInfo) -> ! {
    loop {}
}
