pub fn log_error_with<T: std::fmt::Display>(message: &str, err: T) {
    eprintln!("{message}: {err}");
}

pub fn log_info_with<T: std::fmt::Display>(message: &str, value: T) {
    println!("{message}: {value}");
}
