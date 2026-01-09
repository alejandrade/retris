/// Cross-platform logging that works on both native and WASM/browser

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Log a message to the console (works on both native and browser)
#[cfg(target_arch = "wasm32")]
#[allow(dead_code)]
pub fn log(message: &str) {
    web_sys::console::log_1(&JsValue::from_str(message));
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn log(message: &str) {
    println!("{}", message);
}

/// Log a formatted message (works on both native and browser)
#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        $crate::logger::log(&format!($($arg)*))
    };
}
