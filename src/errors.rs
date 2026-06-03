use serde::Serialize;
use worker::{Response, ResponseBuilder, console_error, console_warn};

// Custom error type that maps to specific errors this application may encounter.
#[derive(Debug)]
pub enum AppError {
    Internal { error: String },
    InsufficientSources,
    UnsupportedValue { param: &'static str, value: String, supported: &'static [&'static str] },
    RequiredParameter { param: &'static str }
}

// Client facing representation of an error, following RFC 9457.
#[derive(Serialize)]
struct ProblemDetails {
    title: &'static str,
    status: u16,
    detail: String
}

// Conversion from internal to external representation.
impl From<AppError> for ProblemDetails {
    fn from(e: AppError) -> Self {
        match e {
            AppError::InsufficientSources => {
                console_warn!("503 Insufficient Sources");
                ProblemDetails {
                    title: "Price Unavailable",
                    status: 503,
                    detail: "Unable to retrieve price data at this time, please try again".into()
                }
            },
            AppError::Internal { error } => { 
                console_error!("500 Internal Server Error: {error}");
                ProblemDetails {
                    title: "Internal Server Error",
                    status: 500,
                    detail: "An unexpected error occurred".into()
                }
            },
            AppError::UnsupportedValue { param, value, supported } => ProblemDetails {
                title: "Invalid Parameter",
                status: 400,
                // TODO: The double quotations are not formatting correctly.
                detail: format!("\"{value}\" is not a supported value for \"{param}\", accepted values are: {}", supported.join(", "))
            },
            AppError::RequiredParameter { param } => ProblemDetails {
                title: "Required Parameter",
                status: 400,
                detail: format!("Missing required parameter: {param}")
            }
        }
    }
}

// Builds a response containing ProblemDetails from an AppError.
impl AppError {
    // Uses `expect()` as these are unrecoverable errors. These should only trigger if the runtime itself is broken.
    pub fn into_response(self) -> Response {
        let problem: ProblemDetails = self.into();
        
        let mut response = ResponseBuilder::new()
            .with_status(problem.status)
            .from_json(&problem)
            .expect("Failed to build error response");

        response.headers_mut()
            .set("Content-Type", "application/problem+json")
            .expect("Failed to set error response headers");

        response
    }
}

impl From<worker::Error> for AppError {
    fn from(e: worker::Error) -> Self {
        AppError::Internal { error: e.to_string() }
    }
}

impl From<worker::KvError> for AppError {
    fn from(e: worker::KvError) -> Self {
        AppError::Internal { error: e.to_string() }
    }
}

impl From<serde_json::Error> for AppError {
    fn from(e: serde_json::Error) -> Self {
        AppError::Internal { error: e.to_string() }
    }
}