#![forbid(unsafe_code)]
#![deny(clippy::all)]

// Reexport crate
pub use backtrace;

pub trait BacktraceFilter {
    /// Filter out all frame symbols that you do or don't want in the backtrace list.
    /// Only the items that return `true` will be kept.
    fn filter(symbol: &backtrace::Symbol) -> bool;

    /// Get info about the last stack trace
    fn get_backtrace_info(mut amount: u16) -> String {
        let mut bt_info = vec![];
        backtrace::trace(|frame| {
            backtrace::resolve_frame(frame, |symbol| {
                // filter out all symbols we do not want.
                if !Self::filter(symbol) {
                    // Skip symbol
                    return;
                }

                if let Some(name) = symbol.name() {
                    let name = name.to_string();
                    // Check if part of our code
                    if name.starts_with("myemma_backtrace::get_backtrace_info") {
                        // Not part of our code so lets skip
                        return;
                    }

                    bt_info.push(format!(
                        "   {} at {}:{}{}",
                        name,
                        symbol
                            .filename()
                            .map(|filename| filename.display().to_string())
                            .unwrap_or(String::new()),
                        symbol
                            .lineno()
                            .map(|lineno| lineno.to_string())
                            .unwrap_or(String::new()),
                        symbol
                            .colno()
                            .map(|colno| format!(":{}", colno))
                            .unwrap_or(String::new())
                    ));
                    amount -= 1;
                }
            });
            amount != 0 // keep going to the next frame until we have the amount we want
        });

        if !bt_info.is_empty() {
            format!("\n{}", bt_info.join("\n"))
        } else {
            "".to_owned()
        }
    }
}
