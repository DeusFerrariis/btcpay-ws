use colored::Colorize;

pub fn error(string: String) {
    println!("{} {}", "[ERROR]".red().bold(), string);
}

pub fn warn(string: String) {
    println!("{} {}", "[WARN]".yellow().bold(), string);
}

pub fn info(string: String) {
    println!("{} {}", "[INFO]".cyan().bold(), string);
}

#[macro_export]
macro_rules! trace {
    ($fmt_str:expr, $($params:expr),*) => {
        println!("[TRACE] {}", format!(fmt_str, $(params,)*));
    }
}
