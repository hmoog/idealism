use std::sync::Weak;

use crate::blocks::{BlockMetadata, block_metadata::BlockMetadataInner};

pub struct BlockMetadataRef(Weak<BlockMetadataInner>);

impl BlockMetadataRef {
    pub fn new(weak: Weak<BlockMetadataInner>) -> Self {
        Self(weak)
    }

    pub fn upgrade(&self) -> Option<BlockMetadata> {
        self.0.upgrade().map(BlockMetadata)
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
            Self::new(Weak::new())
        }
    }

    impl Clone for BlockMetadataRef {
        fn clone(&self) -> Self {
            Self(Weak::clone(&self.0))
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
