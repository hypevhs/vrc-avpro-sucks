#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        eprintln!("[DEBUG][{}:{}] {}", file!(), line!(), format_args!($($arg)*));
    }
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        eprintln!("[ERROR][{}:{}] {}", file!(), line!(), format_args!($($arg)*));
    }
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        eprintln!("[WARN][{}:{}] {}", file!(), line!(), format_args!($($arg)*));
    }
}
