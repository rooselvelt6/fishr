use thiserror::Error;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("Invalid ULID: {0}")]
    InvalidUlid(String),
    #[error("Sync conflict: {0}")]
    SyncConflict(String),
    #[error("Decimal conversion error: {0}")]
    DecimalError(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Validation error: {0}")]
    Validation(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_error_serde_from() {
        let json_err = serde_json::from_str::<()>("invalid").unwrap_err();
        let err: CoreError = json_err.into();
        assert!(err.to_string().contains("Serialization error"));
    }

    #[test]
    fn test_core_error_validation() {
        let err = CoreError::Validation("campo requerido".into());
        assert_eq!(err.to_string(), "Validation error: campo requerido");
    }

    #[test]
    fn test_core_error_not_found() {
        let err = CoreError::NotFound("usuario".into());
        assert_eq!(err.to_string(), "Not found: usuario");
    }

    #[test]
    fn test_core_error_sync_conflict() {
        let err = CoreError::SyncConflict("op_counter ahead".into());
        assert_eq!(err.to_string(), "Sync conflict: op_counter ahead");
    }
}
