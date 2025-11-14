#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    ExcelError(#[from] calamine::Error),
    #[error("Database Config Error: {0}.")]
    DatabaseConfigError(String),
    #[error(transparent)]
    DatabaseError(#[from] sqlx::Error),
    #[error("Excel Config Error: {0}.")]
    ExcelConfigError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
