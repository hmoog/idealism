use std::sync::{Arc, RwLock, RwLockReadGuard};

use common::{
    blocks::{BlockMetadata, BlockMetadataRef},
    extensions::ArcExt,
    rx::Countdown,
    up,
};

pub struct BlockDAGMetadata {
    pub all_parents_available: Countdown,
    parents: RwLock<Vec<BlockMetadataRef>>,
}

impl BlockDAGMetadata {
    pub(crate) fn new(parents_count: usize) -> Self {
        Self {
            parents: RwLock::new(vec![BlockMetadataRef::default(); parents_count]),
            all_parents_available: Countdown::new(parents_count),
        }
    }

    pub(crate) fn register_parent(self: &Arc<Self>, index: usize, parent: &BlockMetadata) {
        self.parents.write().unwrap()[index] = parent.downgrade();

        parent.attach::<Arc<BlockDAGMetadata>>({
            let this = self.downgrade();
            |parent| {
                parent
                    .all_parents_available
                    .attach(move |_| up!(this: this.all_parents_available.decrease()))
            }
        });
    }
}

impl BlockDAGMetadata {
    pub fn parents(&self) -> RwLockReadGuard<Vec<BlockMetadataRef>> {
        self.parents.read().unwrap()
    }
}
