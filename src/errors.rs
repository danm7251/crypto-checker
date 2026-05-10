use serde::Serialize;
use worker::{Headers, Response, ResponseBuilder, console_error, console_warn};

pub enum AppError {
    Internal { error: String },
    InsufficientSources,
    UnsupportedValue { param: &'static str, value: String, supported: &'static [&'static str] },
    RequiredParameter { param: &'static str }
}

#[derive(Serialize)]
struct ProblemDetails {
    title: &'static str,
    status: u16,
    detail: String
}

impl From<AppError> for ProblemDetails {
    fn from(error: AppError) -> Self {
        match error {
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

impl AppError {
    // Uses `expect()` as these are unrecoverable errors. These should only trigger if the runtime itself is broken.
    pub fn into_response(self) -> Response {
        let headers = Headers::new();
        headers.set("Content-Type", "application/problem+json").expect("Failed to set headers");

        let problem: ProblemDetails = self.into();
        //TODO: Header is still application/json, status is still 200 OK.
        ResponseBuilder::new().with_headers(headers).from_json(&problem).expect("Failed to build error")
    }
}

pub trait IntoInternal<T> {
    fn or_internal_error(self) -> Result<T, AppError>;
}

impl<T, E: ToString> IntoInternal<T> for Result<T, E> {
    fn or_internal_error(self) -> Result<T, AppError> {
        self.map_err(|e| AppError::Internal { error: e.to_string() })
    }
}