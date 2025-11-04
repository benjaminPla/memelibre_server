#[macro_export]
macro_rules! http_error {
    ($status:expr) => {{
        use axum::http::StatusCode;
        let message = match $status {
            StatusCode::INTERNAL_SERVER_ERROR => "Internal server error",
            StatusCode::NOT_FOUND => "Not found",
            StatusCode::PAYLOAD_TOO_LARGE => "Request payload too large",
            StatusCode::UNAUTHORIZED => "Unauthorized",
            _ => "Internal server error",
        };
        eprintln!(
            "{}:{} - HTTP {} {}",
            file!(),
            line!(),
            $status.as_u16(),
            message
        );
        ($status, message.to_string())
    }};

    ($status:expr, $custom_msg:expr) => {{
        eprintln!(
            "{}:{} - HTTP {} {}",
            file!(),
            line!(),
            $status.as_u16(),
            $custom_msg
        );
        ($status, $custom_msg.to_string())
    }};

    ($status:expr, err: $error:expr) => {{
        eprintln!(
            "{}:{} - HTTP {} - {:#?}",
            file!(),
            line!(),
            $status.as_u16(),
            $error
        );
        ($status, "Internal server error".to_string())
    }};
}
