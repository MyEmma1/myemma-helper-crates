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
                if amount == 0 {
                    return;
                }
                // filter out all symbols we do not want.
                if !Self::filter(symbol) {
                    // Skip symbol
                    return;
                }

                let mut frame_info = String::new();
                if let Some(name) = symbol.name() {
                    let name = name.to_string();
                    // Check if part of our code
                    if name.starts_with("myemma_backtrace::get_backtrace_info") {
                        // Not part of our code so lets skip
                        return;
                    }
                    frame_info = name;
                }

                // In debug mode add line numbers
                #[cfg(debug_assertions)]
                if let Some(line) = symbol.lineno() {
                    frame_info = format!("{} line: {}", frame_info, line);
                }

                // Only add if not empty
                if !frame_info.is_empty() {
                    bt_info.push(frame_info);
                    amount -= 1;
                }
            });
            true // keep going to the next frame
        });
        if !bt_info.is_empty() {
            let separator = "\n   at ";
            format!("{}{}", separator, bt_info.join(separator))
        } else {
            "".to_owned()
        }
    }
}
