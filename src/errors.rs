use hyper::StatusCode;
use serde_json::Value;
use stq_http::errors::{Codeable, PayloadCarrier};

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Missing user_id")]
    MissingUserId,
    #[fail(display = "Failed to parse user_id")]
    UserIdParse,
    #[fail(display = "Parse failure")]
    ParseError,
    #[fail(display = "Invalid route")]
    InvalidRoute,
}

impl Codeable for Error {
    fn code(&self) -> StatusCode {
        use self::Error::*;

        match self {
            MissingUserId | UserIdParse { .. } => StatusCode::BadRequest,
            ParseError => StatusCode::UnprocessableEntity,
            InvalidRoute => StatusCode::NotFound,
        }
    }
}

impl PayloadCarrier for Error {
    fn payload(&self) -> Option<Value> {
        None
    }
}
