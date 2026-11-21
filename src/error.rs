//! Error types for the archmanweb component.

/// Errors that can occur indexing or searching man pages.
#[derive(Debug, thiserror::Error)]
pub enum ManwebError {
    /// Pacman db archive could not be read.
    #[error("db error: {0}")]
    Db(String),

    /// I/O error.
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    /// Invalid man page filename.
    #[error("invalid man page: {0}")]
    InvalidManPage(String),
}
