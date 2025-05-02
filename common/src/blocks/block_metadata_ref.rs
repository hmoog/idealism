use std::{backtrace::Backtrace, sync::Weak};

use crate::{
    blocks::{BlockMetadata, block_metadata::BlockMetadataInner},
    errors::Error,
    ids::BlockID,
};

pub struct BlockMetadataRef(BlockID, Weak<BlockMetadataInner>);

impl BlockMetadataRef {
    pub fn new(block_id: BlockID, weak: Weak<BlockMetadataInner>) -> Self {
        Self(block_id, weak)
    }

    pub fn upgrade(&self) -> Option<BlockMetadata> {
        self.1.upgrade().map(BlockMetadata)
    }

    pub fn try_upgrade(&self) -> Result<BlockMetadata, Error> {
        self.1
            .upgrade()
            .map(BlockMetadata)
            .ok_or(Error::BlockNotFound {
                block_id: self.0.clone(),
                backtrace: Backtrace::capture(),
            })
    }
}

mod traits {
    use std::{
        hash::{Hash, Hasher},
        ptr,
        sync::Weak,
    };

    use crate::blocks::BlockMetadataRef;

    impl Default for BlockMetadataRef {
        fn default() -> Self {
            Self::new(Default::default(), Weak::new())
        }
    }

    impl Clone for BlockMetadataRef {
        fn clone(&self) -> Self {
            Self(self.0.clone(), Weak::clone(&self.1))
        }
    }

    impl PartialEq for BlockMetadataRef {
        fn eq(&self, other: &Self) -> bool {
            self.0.as_ptr() == other.0.as_ptr()
        }
    }

    impl Eq for BlockMetadataRef {}

    impl Hash for BlockMetadataRef {
        fn hash<T: Hasher>(&self, state: &mut T) {
            ptr::hash(self.0.as_ptr(), state);
        }
    }
}
