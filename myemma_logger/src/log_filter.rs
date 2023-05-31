use log::Metadata;

pub trait LogFilter {
    /// Filter out all log message that you do or don't want logged.
    /// Only the items that return `true` will be kept.
    fn filter(&self, metadata: &Metadata) -> bool;
}
