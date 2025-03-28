use std::sync::Weak;

use crate::{BlockMetadata, Config, Inner};

pub struct BlockMetadataRef<C: Config>(pub(crate) Weak<Inner<C>>);

impl<C: Config> BlockMetadataRef<C> {
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

    use crate::{BlockMetadataRef, Config};

    impl<C: Config> Default for BlockMetadataRef<C> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<C: Config> Clone for BlockMetadataRef<C> {
        fn clone(&self) -> Self {
            Self(Weak::clone(&self.0))
        }
    }

    impl<C: Config> PartialEq for BlockMetadataRef<C> {
        fn eq(&self, other: &Self) -> bool {
            self.0.as_ptr() == other.0.as_ptr()
        }
    }

    impl<C: Config> Eq for BlockMetadataRef<C> {}

    impl<C: Config> Hash for BlockMetadataRef<C> {
        fn hash<T: Hasher>(&self, state: &mut T) {
            ptr::hash(self.0.as_ptr(), state);
        }
    }
}
