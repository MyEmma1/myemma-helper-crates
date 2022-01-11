#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ApiErrorKind {
    /// An internal error
    /// HTTP Status code: 500
    ServerError,
    /// HTTP Status code: 400
    BadRequest,
    /// HTTP Status code: 401
    Unauthorized,
    /// HTTP Status code: 403
    Forbidden,
    /// HTTP Status code: 404
    NotFound,
    /// HTTP Status code: 422
    UnprocessableEntity,
    /// Error message that never leaves server.
    /// HTTP Status code: 500
    PrivateError,
}

impl Default for ApiErrorKind {
    fn default() -> Self {
        ApiErrorKind::PrivateError
    }
}

impl ApiErrorKind {
    pub fn is_server_error(&self) -> bool {
        *self == ApiErrorKind::ServerError || *self == ApiErrorKind::PrivateError
    }
}
