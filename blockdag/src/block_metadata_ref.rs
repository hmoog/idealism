use std::sync::Weak;

use crate::{BlockDAGConfig, BlockMetadata, Inner};

pub struct BlockMetadataRef<C: BlockDAGConfig>(pub(crate) Weak<Inner<C>>);

impl<C: BlockDAGConfig> BlockMetadataRef<C> {
    pub fn new() -> Self {
        Self(Weak::new())
    }

    pub fn upgrade(&self) -> Option<BlockMetadata<C>> {
        self.0.upgrade().map(BlockMetadata)
    }
}

mod traits {
    use std::{
        hash::{Hash, Hasher},
        ptr,
        sync::Weak,
    };

    use crate::{BlockDAGConfig, BlockMetadataRef};

    impl<C: BlockDAGConfig> Default for BlockMetadataRef<C> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<C: BlockDAGConfig> Clone for BlockMetadataRef<C> {
        fn clone(&self) -> Self {
            Self(Weak::clone(&self.0))
        }
    }

    impl<C: BlockDAGConfig> PartialEq for BlockMetadataRef<C> {
        fn eq(&self, other: &Self) -> bool {
            self.0.as_ptr() == other.0.as_ptr()
        }
    }

    impl<C: BlockDAGConfig> Eq for BlockMetadataRef<C> {}

    impl<C: BlockDAGConfig> Hash for BlockMetadataRef<C> {
        fn hash<T: Hasher>(&self, state: &mut T) {
            ptr::hash(self.0.as_ptr(), state);
        }
    }
}
