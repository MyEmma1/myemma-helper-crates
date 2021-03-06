#![forbid(unsafe_code)]
#![deny(clippy::all)]

use chrono::Utc;
use google_cloud_logging::*;
use log::{Level, Metadata, Record};
use myemma_backtrace::BacktraceFilter;
use std::marker::PhantomData;
/// Re-export Yansi Paint so painter can be disabled: `Paint::disable();`
pub use yansi::Paint;

/// The log collector and handler for most printed messages in terminal.
#[derive(Debug)]
pub struct Logger<B, L>
where
    B: BacktraceFilter + Sized + Send + Sync,
    L: LogFilter + Sized + Send + Sync + Default,
{
    format: LogFormat,
    backtrace_count: u16,
    log_filter: L,
    _backtrace_filter: PhantomData<B>,
}

#[derive(Debug)]
pub enum LogFormat {
    Text,
    Json,
}

impl LogFormat {
    pub fn get_format() -> Self {
        let log_format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "text".to_owned());
        match log_format.as_ref() {
            "json" => LogFormat::Json,
            _ => LogFormat::Text,
        }
    }
}

impl Default for LogFormat {
    fn default() -> Self {
        Self::Text
    }
}

impl<B, L> Default for Logger<B, L>
where
    B: BacktraceFilter + Sized + Send + Sync,
    L: LogFilter + Sized + Send + Sync + Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<B, L> Logger<B, L>
where
    B: BacktraceFilter + Sized + Send + Sync,
    L: LogFilter + Sized + Send + Sync + Default,
{
    pub fn new() -> Self {
        Self {
            format: LogFormat::get_format(),
            backtrace_count: 4,
            log_filter: L::default(),
            _backtrace_filter: PhantomData,
        }
    }

    pub fn new_with_count(backtrace_count: u16) -> Self {
        Self {
            format: LogFormat::get_format(),
            backtrace_count,
            log_filter: L::default(),
            _backtrace_filter: PhantomData,
        }
    }
}

pub trait LogFilter {
    /// Filter out all log message that you do or don't want logged.
    /// Only the items that return `true` will be kept.
    fn filter(&self, metadata: &Metadata) -> bool;
}

impl<B, L> log::Log for Logger<B, L>
where
    B: BacktraceFilter + Sized + Send + Sync,
    L: LogFilter + Sized + Send + Sync + Default,
{
    fn enabled(&self, metadata: &Metadata) -> bool {
        self.log_filter.filter(metadata)
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut level = record.level();
            // Convert Rocket launch warnings to info
            // https://github.com/SergioBenitez/Rocket/issues/1828
            if level == Level::Warn
                && (record.target().eq("rocket::launch_") || record.target().eq("rocket::launch"))
            {
                level = Level::Info;
            }

            match self.format {
                LogFormat::Text => {
                    println!(
                        "{:<5}:{} - {}{}",
                        match level {
                            Level::Error => Paint::red("ERROR"),
                            Level::Warn => Paint::yellow("WARN"),
                            Level::Info => Paint::blue("INFO"),
                            Level::Debug => Paint::green("DEBUG"),
                            Level::Trace => Paint::magenta("TRACE"),
                        },
                        Paint::new(record.target()).dimmed(),
                        record.args(),
                        match level {
                            Level::Error | Level::Warn =>
                                B::get_backtrace_info(self.backtrace_count),
                            _ => "".to_owned(),
                        }
                    );
                }
                LogFormat::Json => {
                    // Once Rocket has updated logging more info can be provided:
                    // https://github.com/SergioBenitez/Rocket/issues/21
                    // https://github.com/SergioBenitez/Rocket/pull/1579
                    let log_entry = GoogleCloudStructLog {
                        severity: Some(match level {
                            Level::Error => GCLogSeverity::Error,
                            Level::Warn => GCLogSeverity::Warning,
                            Level::Info => GCLogSeverity::Info,
                            Level::Debug => GCLogSeverity::Debug,
                            Level::Trace => GCLogSeverity::Default,
                        }),
                        report_type: match level {
                            // More info see: https://cloud.google.com/error-reporting/docs/formatting-error-messages#@type
                            Level::Error => Some("type.googleapis.com/google.devtools.clouderrorreporting.v1beta1.ReportedErrorEvent".to_owned()),
                            _ => None,
                        },
                        message: Some(
                            format!(
                                "{}{}", 
                                record.args(),
                                B::get_backtrace_info(self.backtrace_count)
                            )
                        ),
                        operation: Some(GCOperation {
                            id: Some("MyEmma Backend"),
                            producer: Some("MyEmma Backend Core"),
                            ..Default::default()
                        }),
                        source_location: Some(GCSourceLocation {
                            file: record.file_static(),
                            line: record.line().map(|s| s.to_string()),
                            function: record.module_path_static(),
                        }),
                        time: Some(Utc::now()),
                        ..Default::default()
                    };
                    println!(
                        "{}",
                        serde_json::to_string(&log_entry).expect("Error during logging")
                    );
                }
            }
        }
    }

    fn flush(&self) {}
}
