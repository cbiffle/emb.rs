extern crate core;

/// This will be invoked on `panic!`.  Applications can override this by
/// adding the 'app_panic_fmt' feature.
#[cfg(not(feature = "app_panic_fmt"))]
#[lang = "panic_fmt"]
pub extern fn panic_fmt(_msg: core::fmt::Arguments,
                        _file: &'static str,
                        _line: u32) -> ! {
    loop {}
}
