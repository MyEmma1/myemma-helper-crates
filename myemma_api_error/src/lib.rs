#![forbid(unsafe_code)]
#![deny(clippy::all)]

mod error_kind;
mod error_manipulation;
mod from_std;

pub use error_kind::ApiErrorKind;
pub use error_manipulation::ApiErrorManipulation;
use std::default::Default;
use std::fmt::Debug;
use backtrace::Backtrace;

/// Represents all errors that may occur in the application (server).
/// These errors will not be returned to the user, but they can be converted.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct ApiError<C> {
    /// Contains the error message
    msg: String,
    /// The type of error message. Will effect HTTP status when converted
    pub kind: ApiErrorKind,
    /// The api error message code, might be returned to used depending on `kind`.
    code: C,
    /// Unique error id
    unique_id: String,
    /// Backtrace
    backtrace: Backtrace
}

impl<C: PartialEq> PartialEq for ApiError<C> {
    fn eq(&self, other: &Self) -> bool {
        self.msg == other.msg && self.kind == other.kind && self.code == other.code && self.unique_id == other.unique_id
    }
}

impl<C> ApiError<C>
where
    C: Default + Debug + Clone + PartialEq,
    u16: From<C>,
{
    #[must_use]
    pub fn new<S: AsRef<str>>(msg: S, kind: ApiErrorKind, code: C) -> Self {
        let mut new_internal_error = Self {
            msg: msg.as_ref().to_owned(),
            kind,
            code,
            unique_id: "".to_owned(),
            backtrace: Backtrace::new_unresolved()
        };
        Self::create_new_issue_id(&mut new_internal_error);
        new_internal_error
    }

    #[must_use]
    pub fn new_private<S: AsRef<str>>(msg: S) -> Self {
        log::error!("Private error {}", msg.as_ref());
        Self::new(msg, ApiErrorKind::PrivateError, C::default())
    }

    #[must_use]
    pub fn new_unauthorized<S: AsRef<str>>(msg: S) -> Self {
        Self::new(msg, ApiErrorKind::Unauthorized, C::default())
    }

    #[must_use]
    pub fn new_by_kind(kind: ApiErrorKind) -> Self {
        Self::new("", kind, C::default())
    }

    #[must_use]
    pub fn new_by_code(code: C) -> Self {
        Self::new("", ApiErrorKind::ServerError, code)
    }

    #[must_use]
    pub fn new_internal_server_error() -> Self {
        Self::new(
            "The server had an internal error.",
            ApiErrorKind::ServerError,
            C::default(),
        )
    }

    #[must_use]
    pub fn get_backtrace(&self) -> Backtrace {
        let mut backtrace = self.backtrace.clone();
        backtrace.resolve();
        backtrace
    }

    #[must_use]
    pub fn get_msg_for_internal_use(&self) -> String {
        self.msg.clone()
    }

    #[must_use]
    pub fn get_msg_for_public_use(&self) -> String {
        if self.kind == ApiErrorKind::PrivateError {
            "Internal server error".to_owned()
        } else {
            self.msg.clone()
        }
    }

    #[must_use]
    pub fn get_kind(&self) -> ApiErrorKind {
        self.kind
    }

    pub fn set_kind(&mut self, kind: ApiErrorKind) {
        log::info!(
            "Changed internal error ({}): kind: `{:?}`->`{:?}`",
            self.unique_id,
            self.kind,
            kind,
        );
        self.kind = kind;
    }

    #[must_use]
    pub fn get_raw_code_for_internal_use(&self) -> C {
        self.code.clone()
    }

    #[must_use]
    pub fn get_code_for_internal_use(&self) -> u16 {
        self.code.clone().into()
    }

    #[must_use]
    pub fn get_code_for_public_use(&self) -> u16 {
        if self.kind == ApiErrorKind::PrivateError {
            0
        } else {
            self.code.clone().into()
        }
    }

    /// Get the unique id for this issue, can be used to link logs together.
    #[must_use]
    pub fn get_unique_id(&self) -> String {
        self.unique_id.clone()
    }

    /// Get the unique id for this issue in format that can be printed to log.
    #[must_use]
    pub fn log_link(&self) -> String {
        format!("(Error ID: {})", self.unique_id)
    }

    #[must_use]
    pub fn display_error_for_internal_use(&self) -> String {
        format!(
            "Internal Error (ID: `{}`, Code: `{}`, Kind: `{:?}`): {}",
            self.unique_id,
            self.get_code_for_internal_use(),
            self.kind,
            self.get_msg_for_internal_use()
        )
    }

    /// Conditionally change internal error when code matched with `self`.
    #[must_use]
    pub fn transform_on_code(self, code: C, other: Self) -> Self {
        if self.code == code {
            Self::transform_to(self, other)
        } else {
            self
        }
    }

    /// Change internal error, but keep unique id from `self`.
    #[must_use]
    pub fn transform_to(self, mut other: Self) -> Self {
        // Check what parts of the error have changed.
        let mut changed = String::new();
        // Check if other error has an existing id
        if !other.unique_id.is_empty() {
            // Because id is getting replaced the value of other is getting replaced with `self`
            changed.push_str(&format!(
                "\n- id  : `{}`->`{}`",
                other.unique_id, self.unique_id
            ));
        }
        if self.msg != other.msg {
            changed.push_str(&format!("\n- msg : `{}`->`{}`", self.msg, other.msg));
        }
        if self.kind != other.kind {
            changed.push_str(&format!("\n- kind: `{:?}`->`{:?}`", self.kind, other.kind));
        }
        if self.code != other.code {
            changed.push_str(&format!("\n- code: `{:?}`->`{:?}`", self.code, other.code));
        }
        // In case nothing changed, make this clear
        if changed.is_empty() {
            changed.push_str("No values changed");
        }
        // Replace unique_id
        other.unique_id = self.unique_id;
        log::info!(
            "Transforming internal error ({}): {}",
            other.unique_id,
            changed
        );
        other
    }

    /// Transform the error to new code, changes are logged.
    #[must_use]
    pub fn transform_code_only(self, code: C) -> Self {
        let mut new_error = self.clone();
        // Replace code
        new_error.code = code;
        self.transform_to(new_error)
    }

    /// Create a new hash
    fn create_new_issue_id(&mut self) {
        if !self.unique_id.is_empty() {
            log::error!("InternalError already has an issue id.");
            return;
        }

        let amount_chars: usize = 20;
        use rand::Rng;
        let hash: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(amount_chars)
            .map(char::from)
            .collect();
        self.unique_id = hash;
    }
}

impl<C> std::fmt::Display for ApiError<C>
where
    C: Default + Debug + Clone + PartialEq,
    u16: From<C>,
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            formatter,
            "API Error (ID: `{}`): {}",
            self.get_unique_id(),
            self.get_msg_for_public_use(),
        )
    }
}

impl<C> std::error::Error for ApiError<C>
where
    C: Default + Debug + Clone + PartialEq,
    u16: From<C>,
{
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Copy, Clone)]
    #[repr(u16)]
    pub enum ApiErrorCodes {
        /// Default error. `#0`
        Default = 0,
    }

    impl From<ApiErrorCodes> for u16 {
        fn from(code: ApiErrorCodes) -> Self {
            code as u16
        }
    }

    impl Default for ApiErrorCodes {
        fn default() -> Self {
            Self::Default
        }
    }

    #[test]
    fn test_new_error() {
        let error_msg = "label name";
        let error = ApiError::<ApiErrorCodes>::new_private(error_msg);
        assert_eq!(
            ApiError::<ApiErrorCodes> {
                msg: error_msg.to_owned(),
                kind: ApiErrorKind::PrivateError,
                code: ApiErrorCodes::Default,
                unique_id: error.get_unique_id(),
                backtrace: Backtrace::new_unresolved()
            },
            error
        );
    }

    #[test]
    fn test_error_display() {
        let error = ApiError::<ApiErrorCodes>::new_private("error message");
        assert_eq!(
            format!(
                "API Error (ID: `{}`): Internal server error",
                error.get_unique_id()
            ),
            format!("{}", error)
        );
    }
}
