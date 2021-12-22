#![forbid(unsafe_code)]
#![deny(clippy::all)]

/// Provide custom error with links for when application panics (unrecoverable error).
#[macro_export]
macro_rules! setup_panic_hook {
    (
        $operation_identifier:literal, $producer_identifier:literal,
        $backtrace_filter:ty, $backtrace_count:literal $(,)*
    ) => {
        use std::panic::{self, PanicInfo};

        // This code is inspired by the `human-panic` crate.
        // Only use custom panic when `RUST_BACKTRACE` is not set.
        match ::std::env::var("RUST_BACKTRACE") {
            Err(_) => {
                panic::set_hook(Box::new(move |info: &PanicInfo| {
                    use google_cloud_logging::*;
                    use chrono::{Utc};

                    let payload = info.payload();
                    let panic_message = if let Some(s) = payload.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = payload.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        String::new()
                    };
                    // Can also parse other objects like `InternalError`
                    // But this is never return because `unwrap()` converts it to a string.
                    // https://doc.rust-lang.org/std/panic/fn.panic_any.html
                    // if let Some(s) = payload.downcast_ref::<InternalError>() {
                    //     s.to_string()
                    // }

                    let mut file = "unknown file";
                    let mut line = 0;
                    // let mut column = 0;
                    if let Some(location) = info.location() {
                        file = location.file();
                        line = location.line();
                        // column = location.column();
                    }

                    match myemma_logger::LogFormat::get_format() {
                        myemma_logger::LogFormat::Text => {
                            println!(
                                "PANIC:{} - {}:{}{}",
                                panic_message,
                                file,
                                line,
                                <$backtrace_filter>::get_backtrace_info($backtrace_count),
                            );
                        }
                        myemma_logger::LogFormat::Json => {
                            let log_entry = GoogleCloudStructLog {
                                severity: Some(GCLogSeverity::Critical),
                                // More info see: https://cloud.google.com/error-reporting/docs/formatting-error-messages#@type
                                report_type: Some(
                                    "type.googleapis.com/google.devtools.clouderrorreporting.v1beta1.ReportedErrorEvent"
                                    .to_owned()
                                ),
                                message: Some(
                                    format!("{}{}", panic_message, <$backtrace_filter>::get_backtrace_info($backtrace_count))
                                ),
                                operation: Some(GCOperation {
                                    id: Some($operation_identifier),
                                    producer: Some($producer_identifier),
                                    ..Default::default()
                                }),
                                source_location: Some(GCSourceLocation {
                                    file: Some(file),
                                    line: Some(line.to_string()),
                                    function: None,
                                }),
                                time: Some(Utc::now()),
                                ..Default::default()
                            };
                            println!(
                                "{}",
                                serde_json::to_string(&log_entry).expect("Error during logging panic")
                            );
                        }
                    }
                }));
            }
            Ok(_) => {}
        }
    };
}
