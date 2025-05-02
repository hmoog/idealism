use std::backtrace::Backtrace;
use std::fmt;
use crate::ids::BlockID;

#[derive(Debug)]
pub enum Error {
    BlockNotFound {
        block_id: BlockID,
        backtrace: Backtrace,
    },

    MetadataNotFound {
        block_id: BlockID,
        metadata: &'static str,
        backtrace: Backtrace,
    },
}
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::BlockNotFound { block_id, backtrace } => {
                write!(f, "Block `{}` not found`\nBacktrace:\n{}", block_id, backtrace)
            }
            Error::MetadataNotFound {
                block_id,
                metadata,
                backtrace,
            } => {
                write!(
                    f,
                    "Metadata `{}` not found in block `{}`\nBacktrace:\n{}",
                    metadata, block_id, backtrace
                )
            }
        }
    }
}

impl std::error::Error for Error {}