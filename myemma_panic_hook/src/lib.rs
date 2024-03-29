#![forbid(unsafe_code)]
#![deny(clippy::all)]

pub extern crate chrono;
pub extern crate google_cloud_logging;
pub extern crate myemma_logger;
pub extern crate serde_json;

/// Provide custom error with links for when application panics (unrecoverable error).
#[macro_export]
macro_rules! setup_panic_hook {
    (
        $operation_identifier:literal, $producer_identifier:literal,
        $backtrace_filter:ty, $backtrace_count:literal $(,)*
    ) => {
        // This code is inspired by the `human-panic` crate.
        // Only use custom panic when `RUST_BACKTRACE` is not set.
        match ::std::env::var("RUST_BACKTRACE") {
            Err(_) => {
                std::panic::set_hook(Box::new(move |info: &std::panic::PanicInfo| {
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

                    match $crate::myemma_logger::LogFormat::get_format() {
                        $crate::myemma_logger::LogFormat::Text => {
                            println!(
                                "PANIC:{} - {}:{}{}",
                                panic_message,
                                file,
                                line,
                                <$backtrace_filter>::get_backtrace_info($backtrace_count),
                            );
                        }
                        $crate::myemma_logger::LogFormat::Json => {
                            let log_entry = $crate::google_cloud_logging::GoogleCloudStructLog {
                                severity: Some($crate::google_cloud_logging::GCLogSeverity::Critical),
                                // More info see: https://cloud.google.com/error-reporting/docs/formatting-error-messages#@type
                                report_type: Some(
                                    "type.googleapis.com/google.devtools.clouderrorreporting.v1beta1.ReportedErrorEvent"
                                    .to_owned()
                                ),
                                message: Some(
                                    format!("{}{}", panic_message, <$backtrace_filter>::get_backtrace_info($backtrace_count))
                                ),
                                operation: Some($crate::google_cloud_logging::GCOperation {
                                    id: Some($operation_identifier),
                                    producer: Some($producer_identifier),
                                    ..Default::default()
                                }),
                                source_location: Some($crate::google_cloud_logging::GCSourceLocation {
                                    file: Some(file),
                                    line: Some(line.to_string()),
                                    function: None,
                                }),
                                time: Some($crate::chrono::Utc::now()),
                                ..Default::default()
                            };
                            println!(
                                "{}",
                                $crate::serde_json::to_string(&log_entry).expect("Error during logging panic")
                            );
                        }
                    }
                }));
            }
            Ok(_) => {}
        }
    };
}
