use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Block not found")]
    BlockNotFound,

    #[error("Metadata not found for type `{0}`")]
    MetadataNotFound(&'static str),
}
