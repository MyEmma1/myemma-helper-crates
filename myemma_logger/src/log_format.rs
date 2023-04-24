#[derive(Debug, Default)]
pub enum LogFormat {
    #[default]
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