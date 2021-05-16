#[macro_use]
pub mod macros {
    pub use utils;
    use colored::Colorize;
    use std::fmt;
    use std::env;

    #[derive(PartialEq)]
    pub enum LogLevel {
        Trace,
        Debug,
        Info,
        Warn,
        Error
    }

    impl LogLevel {
        pub fn visible(&self, other: &Self) -> bool {
            match self {
                LogLevel::Error => other == &LogLevel::Error,
                LogLevel::Warn => vec![LogLevel::Error, LogLevel::Warn].contains(other),
                LogLevel::Info => vec![LogLevel::Error, LogLevel::Warn, LogLevel::Info].contains(other),
                LogLevel::Debug => vec![LogLevel::Error, LogLevel::Warn, LogLevel::Info, LogLevel::Debug].contains(other),
                LogLevel::Trace => true
            }
        }
    }

    impl fmt::Display for LogLevel {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                LogLevel::Trace => write!(f, "{}", "trace:".blue().bold()),
                LogLevel::Debug => write!(f, "{}", "debug:".green().bold()),
                LogLevel::Info => write!(f, "{}", "info: ".cyan().bold()),
                LogLevel::Warn => write!(f, "{}", "warn: ".yellow().bold()),
                LogLevel::Error => write!(f, "{}", "error:".red().bold()),
            }
        }
    }

    pub fn get_log_level() -> LogLevel {
        match env::var("BTCPAY_WS_LOGLEVEL") {
            Ok(val) => match val.as_str() {
                "trace" => LogLevel::Trace,
                "debug" => LogLevel::Debug,
                "info" => LogLevel::Info,
                "warn" => LogLevel::Warn,
                _ => LogLevel::Error
            },
            Err(_) => LogLevel::Error
        }
    }

    #[macro_export]
    macro_rules! trace {
        ($fmt_str:literal, $($params:expr),*) => {
            if $crate::macros::get_log_level()
                .visible(&$crate::macros::LogLevel::Trace) {
                println!(
                    "{} [{}] {}",
                    $crate::macros::LogLevel::Trace,
                    $crate::macros::utils::function!(),
                    format!($fmt_str, $($params,)*)
                );
            }
        };

        ($fmt_str:literal) => {
            if $crate::macros::get_log_level()
                .visible(&$crate::macros::LogLevel::Trace) {
                println!(
                    "{} [{}] {}",
                    $crate::macros::LogLevel::Trace,
                    $crate::macros::utils::function!(),
                    $fmt_str
                );
            }
        };
    }

    #[macro_export]
    macro_rules! info {
        ($fmt_str:literal, $($params:expr),*) => {
            if $crate::macros::get_log_level()
                .visible(&$crate::macros::LogLevel::Info) {
                println!(
                    "{} [{}] {}",
                    $crate::macros::LogLevel::Info,
                    $crate::macros::utils::function!(),
                    format!($fmt_str, $($params,)*)
                );
            }
        };

        ($fmt_str:literal) => {
            if $crate::macros::get_log_level()
                .visible(&$crate::macros::LogLevel::Info) {
                println!(
                    "{} [{}] {}",
                    $crate::macros::LogLevel::Info,
                    $crate::macros::utils::function!(),
                    $fmt_str
                );
            }
        };
    }

    #[macro_export]
    macro_rules! debug {
        ($fmt_str:literal, $($params:expr),*) => {
            if $crate::macros::get_log_level()
                .visible(&$crate::macros::LogLevel::Debug) {
                println!(
                    "{} [{}] {}",
                    $crate::macros::LogLevel::Debug,
                    $crate::macros::utils::function!(),
                    format!($fmt_str, $($params,)*)
                );
            }
        };

        ($fmt_str:literal) => {
            if $crate::macros::get_log_level()
                .visible(&$crate::macros::LogLevel::Debug) {
                println!(
                    "{} [{}] {}",
                    $crate::macros::LogLevel::Debug,
                    $crate::macros::utils::function!(),
                    $fmt_str
                );
            }
        };
    }

    #[macro_export]
    macro_rules! warn {
        ($fmt_str:literal, $($params:expr),*) => {
            if $crate::macros::get_log_level()
                .visible(&$crate::macros::LogLevel::Warn) {
                println!(
                    "{} [{}] {}",
                    $crate::macros::LogLevel::Warn,
                    $crate::macros::utils::function!(),
                    format!($fmt_str, $($params,)*)
                );
            }
        };

        ($fmt_str:literal) => {
            if $crate::macros::get_log_level()
                .visible(&$crate::macros::LogLevel::Warn) {
                println!(
                    "{} [{}] {}",
                    $crate::macros::LogLevel::Warn,
                    $crate::macros::utils::function!(),
                    $fmt_str
                );
            }
        };
    }

    #[macro_export]
    macro_rules! error {
        ($fmt_str:literal, $($params:expr),*) => {
            if $crate::macros::get_log_level()
                .visible(&$crate::macros::LogLevel::Error) {
                println!(
                    "{} [{}] {}",
                    $crate::macros::LogLevel::Error,
                    $crate::macros::utils::function!(),
                    format!($fmt_str, $($params,)*)
                );
            }
        };

        ($fmt_str:literal) => {
            if $crate::macros::get_log_level()
                .visible(&$crate::macros::LogLevel::Error) {
                println!(
                    "{} [{}] {}",
                    $crate::macros::LogLevel::Error,
                    $crate::macros::utils::function!(),
                    $fmt_str
                );
            }
        };
    }
}
