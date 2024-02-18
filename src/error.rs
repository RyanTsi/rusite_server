use axum::{http::StatusCode, response::IntoResponse};

pub type Result<T> = core::result::Result<T, Error>;


#[derive(Debug)]
pub enum Error {
    LoginFail,

    // -- Model error
    TicketDeleteFailIdNotFound { id: u64 },
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        println!("->> {:<12} - {self:?}", "INFO_RES");

        (StatusCode::INTERNAL_SERVER_ERROR, "UNHANDLE_CLIENT_ERROR").into_response()
    }
}