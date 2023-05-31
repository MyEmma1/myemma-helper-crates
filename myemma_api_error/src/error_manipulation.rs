use backtrace::Backtrace;

use crate::{ApiError, ApiErrorKind};
use std::fmt::Debug;

pub trait ApiErrorManipulation<C>
where
    ApiError<C>: From<Self>,
    Self: Sized + From<ApiError<C>> + AsRef<ApiError<C>> + AsMut<ApiError<C>>,
    C: Default + Debug + Clone + PartialEq,
    u16: From<C>,
{
    #[must_use]
    fn new<S: AsRef<str>>(msg: S, kind: ApiErrorKind, code: C) -> Self {
        Self::from(ApiError::<C>::new(msg, kind, code))
    }

    #[must_use]
    fn new_private<S: AsRef<str>>(msg: S) -> Self {
        Self::from(ApiError::<C>::new_private(msg))
    }

    #[must_use]
    fn new_unauthorized<S: AsRef<str>>(msg: S) -> Self {
        Self::from(ApiError::<C>::new_unauthorized(msg))
    }

    #[must_use]
    fn new_by_kind(kind: ApiErrorKind) -> Self {
        Self::from(ApiError::<C>::new_by_kind(kind))
    }

    #[must_use]
    fn new_by_code(code: C) -> Self {
        Self::from(ApiError::<C>::new_by_code(code))
    }

    #[must_use]
    fn new_internal_server_error() -> Self {
        Self::from(ApiError::<C>::new_internal_server_error())
    }

    #[must_use]
    fn get_msg_for_internal_use(&self) -> String {
        self.as_ref().get_msg_for_internal_use()
    }

    #[must_use]
    fn get_msg_for_public_use(&self) -> String {
        self.as_ref().get_msg_for_public_use()
    }

    #[must_use]
    fn get_kind(&self) -> ApiErrorKind {
        self.as_ref().get_kind()
    }

    #[must_use]
    fn get_backtrace(&self) -> Backtrace {
        self.as_ref().get_backtrace()
    }

    fn set_kind(&mut self, kind: ApiErrorKind) {
        ApiError::<C>::set_kind(self.as_mut(), kind);
    }

    #[must_use]
    fn get_raw_code_for_internal_use(&self) -> C {
        self.as_ref().get_raw_code_for_internal_use()
    }

    #[must_use]
    fn get_code_for_internal_use(&self) -> u16 {
        self.as_ref().get_code_for_internal_use()
    }

    #[must_use]
    fn get_code_for_public_use(&self) -> u16 {
        self.as_ref().get_code_for_public_use()
    }

    /// Get the unique id for this issue, can be used to link logs together.
    #[must_use]
    fn get_unique_id(&self) -> String {
        self.as_ref().get_unique_id()
    }

    /// Get the unique id for this issue in format that can be printed to log.
    #[must_use]
    fn log_link(&self) -> String {
        self.as_ref().log_link()
    }

    #[must_use]
    fn display_error_for_internal_use(&self) -> String {
        self.as_ref().display_error_for_internal_use()
    }

    /// Conditionally change internal error when code matched with `self`.
    #[must_use]
    fn transform_on_code(self, code: C, other: Self) -> Self {
        let other_api_error = ApiError::<C>::from(other);
        Self::from(ApiError::<C>::from(self).transform_on_code(code, other_api_error))
    }

    /// Change internal error, but keep unique id.
    #[must_use]
    fn transform_to(self, other: Self) -> Self {
        let other_api_error = ApiError::<C>::from(other);
        Self::from(ApiError::<C>::from(self).transform_to(other_api_error))
    }

    /// Transform the error to new code, changes are logged.
    #[must_use]
    fn transform_code_only(self, code: C) -> Self {
        Self::from(ApiError::<C>::from(self).transform_code_only(code))
    }
}
