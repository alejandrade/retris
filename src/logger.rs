#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
    
    #[wasm_bindgen(js_namespace = console)]
    fn warn(s: &str);
    
    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
    
    #[wasm_bindgen(js_namespace = console)]
    fn info(s: &str);
    
    #[wasm_bindgen(js_namespace = console)]
    fn debug(s: &str);
}

/// Simple logger that writes to browser console
pub struct Logger;

impl Logger {
    /// Log an info message to console
    pub fn info(msg: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            info(msg);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            println!("{}", msg);
        }
    }
    
    /// Log a debug message to console
    pub fn debug(msg: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            debug(msg);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            println!("[DEBUG] {}", msg);
        }
    }
    
    /// Log a warning message to console
    pub fn warn(msg: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            warn(msg);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            eprintln!("[WARN] {}", msg);
        }
    }
    
    /// Log an error message to console
    pub fn error(msg: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            error(msg);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            eprintln!("[ERROR] {}", msg);
        }
    }
    
    /// Log with formatting (like println!)
    pub fn log(msg: &str) {
        #[cfg(target_arch = "wasm32")]
        {
            log(msg);
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            println!("{}", msg);
        }
    }
}

/// Convenience macros for logging
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::logger::Logger::info(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        $crate::logger::Logger::debug(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::logger::Logger::warn(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::logger::Logger::error(&format!($($arg)*));
    };
}

#[macro_export]
macro_rules! log_msg {
    ($($arg:tt)*) => {
        $crate::logger::Logger::log(&format!($($arg)*));
    };
}
