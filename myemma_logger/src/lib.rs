#![forbid(unsafe_code)]
#![deny(clippy::all)]

use chrono::Utc;
use google_cloud_logging::{GCLogSeverity, GCOperation, GCSourceLocation, GoogleCloudStructLog};
use log::{Level, Metadata, Record};
use myemma_backtrace::BacktraceFilter;
use std::marker::PhantomData;
/// Re-export Yansi Paint so painter can be disabled: `Paint::disable();`
pub use yansi::Paint;

mod log_format;
pub use log_format::LogFormat;

mod log_filter;
pub use log_filter::LogFilter;

/// The log collector and handler for most printed messages in terminal.
#[derive(Debug)]
pub struct Logger<B, L>
where
    B: BacktraceFilter + Sized + Send + Sync,
    L: LogFilter + Sized + Send + Sync + Default,
{
    id: String,
    producer: String,
    format: LogFormat,
    backtrace_count: u16,
    log_filter: L,
    _backtrace_filter: PhantomData<B>,
}

impl<B, L> Default for Logger<B, L>
where
    B: BacktraceFilter + Sized + Send + Sync,
    L: LogFilter + Sized + Send + Sync + Default,
{
    fn default() -> Self {
        Self::new("MyEmma Backend", "MyEmma Backend Core")
    }
}

impl<B, L> Logger<B, L>
where
    B: BacktraceFilter + Sized + Send + Sync,
    L: LogFilter + Sized + Send + Sync + Default,
{
    pub fn new(id: &str, producer: &str) -> Self {
        Self {
            id: id.to_string(),
            producer: producer.to_string(),
            format: LogFormat::get_format(),
            backtrace_count: 4,
            log_filter: L::default(),
            _backtrace_filter: PhantomData,
        }
    }

    pub fn with_format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }

    pub fn with_backtrace_count(mut self, backtrace_count: u16) -> Self {
        self.backtrace_count = backtrace_count;
        self
    }

    pub fn with_log_filter(mut self, log_filter: L) -> Self {
        self.log_filter = log_filter;
        self
    }
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
            let level = record.level();
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
                        Paint::new(record.target()).dim(),
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
                            id: Some(&self.id),
                            producer: Some(&self.producer),
                            ..Default::default()
                        }),
                        source_location: Some(GCSourceLocation {
                            file: record.file(),
                            line: record.line().map(|lineno| lineno.to_string()),
                            function: record.module_path(),
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
