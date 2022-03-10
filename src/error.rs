use poem::error::ResponseError;
use poem::http::StatusCode;

#[derive(Debug, thiserror::Error)]
#[error("too many requests")]
pub struct RateLimitError;

impl ResponseError for RateLimitError {
    fn status(&self) -> StatusCode {
        StatusCode::TOO_MANY_REQUESTS
    }
}
