/// Model errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum Error {
    #[error("Not found")]
    NotFound,
    #[error("Ambiguous results ({0:?}) for query")]
    Ambiguous(usize),
}
