use serde::Serialize;
use std::io;

use crate::contract::PUBLIC_CONTRACT_SCHEMA;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, serde::Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    InvalidInput,
    NotFound,
    PermissionDenied,
    InvalidData,
    OperationBusy,
    Timeout,
    AlreadyExists,
    Unsupported,
    IoFailure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ErrorReport {
    pub schema: u32,
    pub code: ErrorCode,
    pub message: String,
    pub retryable: bool,
}

impl ErrorReport {
    pub fn from_io(error: &io::Error) -> Self {
        let (code, retryable) = match error.kind() {
            io::ErrorKind::InvalidInput => (ErrorCode::InvalidInput, false),
            io::ErrorKind::NotFound => (ErrorCode::NotFound, false),
            io::ErrorKind::PermissionDenied => (ErrorCode::PermissionDenied, false),
            io::ErrorKind::InvalidData => (ErrorCode::InvalidData, false),
            io::ErrorKind::WouldBlock => (ErrorCode::OperationBusy, true),
            io::ErrorKind::TimedOut => (ErrorCode::Timeout, true),
            io::ErrorKind::AlreadyExists => (ErrorCode::AlreadyExists, false),
            io::ErrorKind::Unsupported => (ErrorCode::Unsupported, false),
            io::ErrorKind::Interrupted
            | io::ErrorKind::ConnectionRefused
            | io::ErrorKind::ConnectionReset
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::NotConnected => (ErrorCode::IoFailure, true),
            _ => (ErrorCode::IoFailure, false),
        };

        Self {
            schema: PUBLIC_CONTRACT_SCHEMA,
            code,
            message: error.to_string(),
            retryable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ErrorCode, ErrorReport};
    use crate::contract::PUBLIC_CONTRACT_SCHEMA;
    use std::io;

    #[test]
    fn io_errors_map_to_stable_public_codes() {
        let busy = ErrorReport::from_io(&io::Error::new(io::ErrorKind::WouldBlock, "busy"));
        assert_eq!(busy.schema, PUBLIC_CONTRACT_SCHEMA);
        assert_eq!(busy.code, ErrorCode::OperationBusy);
        assert!(busy.retryable);

        let invalid =
            ErrorReport::from_io(&io::Error::new(io::ErrorKind::InvalidInput, "bad input"));
        assert_eq!(invalid.code, ErrorCode::InvalidInput);
        assert!(!invalid.retryable);

        let timeout = ErrorReport::from_io(&io::Error::new(io::ErrorKind::TimedOut, "late"));
        assert_eq!(timeout.code, ErrorCode::Timeout);
        assert!(timeout.retryable);
    }
}
