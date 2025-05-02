use std::{
    any::type_name,
    sync::{Arc, RwLock},
};
use std::backtrace::Backtrace;
use crate::{
    blocks::{Block, BlockMetadataRef},
    collections::AnyMap,
    errors::{Error::MetadataNotFound, Result},
    rx::Signal,
};

pub struct BlockMetadata(pub(crate) Arc<BlockMetadataInner>);

pub struct BlockMetadataInner {
    data: RwLock<AnyMap>,
    pub block: Block,
}

impl BlockMetadata {
    pub fn new(block: Block) -> Self {
        Self(Arc::new(BlockMetadataInner {
            data: RwLock::new(AnyMap::default()),
            block,
        }))
    }

    pub fn try_get<T: Send + Sync + Clone + 'static>(&self) -> Result<T> {
        self.metadata::<T>().value().ok_or(MetadataNotFound {
            block_id: self.block.id().clone(),
            metadata: type_name::<T>(),
            backtrace: Backtrace::capture(),       
        })
    }

    pub fn metadata<T: Send + Sync + 'static>(&self) -> Arc<Signal<T>> {
        if let Some(signal) = self.data.read().unwrap().get::<Arc<Signal<T>>>() {
            return signal.clone();
        }

        self.data
            .write()
            .unwrap()
            .get_or_insert_with::<Arc<Signal<T>>, _>(Arc::default)
            .clone()
    }

    pub fn downgrade(&self) -> BlockMetadataRef {
        BlockMetadataRef::new(self.block.id().clone(), Arc::downgrade(&self.0))
    }
}

mod traits {
    use std::{
        fmt::Debug,
        hash::{Hash, Hasher},
        ops::Deref,
        ptr,
        sync::Arc,
    };

    use crate::blocks::{BlockMetadata, block_metadata::BlockMetadataInner};

    impl Deref for BlockMetadata {
        type Target = BlockMetadataInner;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl Clone for BlockMetadata {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }

    impl PartialEq for BlockMetadata {
        fn eq(&self, other: &Self) -> bool {
            Arc::ptr_eq(&self.0, &other.0)
        }
    }

    impl Eq for BlockMetadata {}

    impl Hash for BlockMetadata {
        fn hash<H: Hasher>(&self, state: &mut H) {
            ptr::hash(Arc::as_ptr(&self.0), state);
        }
    }

    impl Debug for BlockMetadata {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("BlockMetadata")
                .field("block", &self.block)
                .finish()
        }
    }
}
