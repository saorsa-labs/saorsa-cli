use std::path::PathBuf;
use thiserror::Error;

/// Custom error types for Saorsa CLI operations.
/// Currently reserved for future use in more sophisticated error handling.
#[allow(dead_code)]
#[derive(Debug, Error)]
pub enum SaorsaError {
    #[error("I/O error during '{operation}' on '{path:?}': {source}")]
    Io {
        operation: String,
        path: Option<PathBuf>,
        source: std::io::Error,
    },
    #[error("Network error for url '{url}': {source}")]
    Network { url: String, source: reqwest::Error },
}

#[allow(dead_code)]
impl SaorsaError {
    pub fn io(operation: &str, source: std::io::Error) -> Self {
        SaorsaError::Io {
            operation: operation.to_string(),
            path: None,
            source,
        }
    }

    pub fn io_with_context<P: Into<PathBuf>>(
        operation: &str,
        path: P,
        source: std::io::Error,
    ) -> Self {
        SaorsaError::Io {
            operation: operation.to_string(),
            path: Some(path.into()),
            source,
        }
    }

    pub fn network(url: &str, source: reqwest::Error) -> Self {
        SaorsaError::Network {
            url: url.to_string(),
            source,
        }
    }

    pub fn network_with_url<U: Into<String>>(url: U, source: reqwest::Error) -> Self {
        SaorsaError::Network {
            url: url.into(),
            source,
        }
    }
}
