use super::{ApiError, ApiErrorKind};
use std::default::Default;
use std::fmt::Debug;

impl<C> From<std::num::ParseIntError> for ApiError<C>
where
    C: Default + Debug + Clone + PartialEq,
    u16: From<C>,
{
    fn from(err: std::num::ParseIntError) -> Self {
        let int_err = Self::new(
            "Could not parse int.",
            ApiErrorKind::BadRequest,
            C::default(),
        );
        log::error!("Could not parse int: {}. {}", err, int_err.log_link());
        int_err
    }
}

impl<C> From<std::num::TryFromIntError> for ApiError<C>
where
    C: Default + Debug + Clone + PartialEq,
    u16: From<C>,
{
    fn from(err: std::num::TryFromIntError) -> Self {
        let int_err = Self::new(
            "Failed to convert number.",
            ApiErrorKind::ServerError,
            C::default(),
        );
        log::error!("Failed to convert number: {}. {}", err, int_err.log_link());
        int_err
    }
}

/// This conversion is happening because of some function that returns this type of error.
/// This will keep the code consistent in all cases.
impl<C> From<std::convert::Infallible> for ApiError<C>
where
    C: Default + Debug + Clone + PartialEq,
    u16: From<C>,
{
    fn from(err: std::convert::Infallible) -> Self {
        let int_err = Self::new(
            "This error can never happen.",
            ApiErrorKind::ServerError,
            C::default(),
        );
        log::error!("Error can never happen: {}. {}", err, int_err.log_link());
        int_err
    }
}

impl<C> From<std::io::Error> for ApiError<C>
where
    C: Default + Debug + Clone + PartialEq,
    u16: From<C>,
{
    fn from(err: std::io::Error) -> Self {
        let int_err = Self::new("IO Error", ApiErrorKind::ServerError, C::default());
        log::error!("IO error: {}. {}", err, int_err.log_link());
        int_err
    }
}

impl<C> From<core::str::Utf8Error> for ApiError<C>
where
    C: Default + Debug + Clone + PartialEq,
    u16: From<C>,
{
    fn from(err: core::str::Utf8Error) -> Self {
        let int_err = Self::new(
            format!("Utf8Error Error: {}", err),
            ApiErrorKind::BadRequest,
            C::default(),
        );
        log::error!("Utf8Error error: {}. {}", err, int_err.log_link());
        int_err
    }
}

impl<C> From<Box<dyn std::any::Any + Send>> for ApiError<C>
where
    C: Default + Debug + Clone + PartialEq,
    u16: From<C>,
{
    fn from(err: Box<dyn std::any::Any + Send>) -> Self {
        let int_err = Self::new("Internal Error", ApiErrorKind::BadRequest, C::default());
        match err.downcast_ref::<String>() {
            Some(as_string) => {
                log::error!("API Error (panic?): {}. {}", as_string, int_err.log_link());
            }
            None => {
                log::error!("API Error (panic?), unknown data. {}", int_err.log_link());
            }
        }
        int_err
    }
}
