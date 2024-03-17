use std::fs::OpenOptions;
use std::io::Write;

#[macro_export]
macro_rules! logline {
    ($msg:expr) => {{
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("/tmp/nanometers.log")
            .expect("Failed to open log file");

        writeln!(file, "{}", $msg).expect("Failed to write to log file");
    }};
}
