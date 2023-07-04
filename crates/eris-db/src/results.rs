use sqlx::FromRow;
use thiserror::Error as ThisError;

/// Model errors
#[derive(Debug, Clone, ThisError)]
pub enum QueryError {
    #[error("Not found")]
    NotFound,
    #[error("Ambiguous results ({0:?}) for query")]
    Ambiguous(usize),
}

#[derive(Debug, Clone, FromRow)]
pub struct Id<T> {
    pub id: T,
}

/*
#[derive(Debug, Clone, FromRow)]
pub struct InsertString {
    pub id: String,
}
*/
