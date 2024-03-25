use myemma_backtrace::{backtrace, BacktraceFilter};
use myemma_panic_hook::setup_panic_hook;

#[test]
pub fn test_build() {
    setup_panic_hook!(
        "MyEmma Backend",
        "MyEmma Backend Core",
        CoreBacktraceFilter,
        8
    );
}
struct CoreBacktraceFilter;

impl BacktraceFilter for CoreBacktraceFilter {
    fn filter(symbol: &backtrace::Symbol) -> bool {
        if let Some(name) = symbol.name() {
            let name = name.to_string();
            // Check if part of our code
            return name.starts_with("controllers::")
                && !name.starts_with("controllers::server::error")
                || name.starts_with("dtos::")
                || name.starts_with("entities::")
                || name.starts_with("main::")
                || name.starts_with("models::") && !name.starts_with("models::error")
                || name.starts_with("services::");
        }
        false
    }
}
