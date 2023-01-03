use std::error::Error;
use std::fmt;

use actix_web::body::BoxBody;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};

#[derive(Debug)]
pub struct WriteConcernNotSatisfiedError {
    required: usize,
    received: usize,
}

impl WriteConcernNotSatisfiedError {
    pub fn new(required: usize, received: usize) -> WriteConcernNotSatisfiedError {
        WriteConcernNotSatisfiedError { required, received }
    }
}

impl fmt::Display for WriteConcernNotSatisfiedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Write Concern Not Satisfied. Required: {}, Received: {}",
            self.required, self.received
        )
    }
}

impl Error for WriteConcernNotSatisfiedError {}

impl ResponseError for WriteConcernNotSatisfiedError {
    fn status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn error_response(&self) -> HttpResponse<BoxBody> {
        let body = format!("{}", self);
        let res = HttpResponse::new(self.status_code());
        res.set_body(BoxBody::new(body))
    }
}
