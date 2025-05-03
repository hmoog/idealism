use std::sync::{Arc, RwLock, RwLockReadGuard};

use common::{
    blocks::{BlockMetadata, BlockMetadataRef},
    rx::Countdown,
};

pub struct BlockDAGMetadata {
    parents: RwLock<Vec<BlockMetadataRef>>,
    pub all_parents_available: Arc<Countdown>,
}

impl BlockDAGMetadata {
    pub fn new(parents_count: usize) -> Self {
        Self {
            parents: RwLock::new(vec![BlockMetadataRef::default(); parents_count]),
            all_parents_available: Arc::new(Countdown::new(parents_count)),
        }
    }

    pub fn set_parent_available(self: &Arc<Self>, index: usize, parent: &BlockMetadata) {
        self.parents.write().unwrap()[index] = parent.downgrade();

        parent.attach({
            let weak_all_parents_available = Arc::downgrade(&self.all_parents_available);
            |block_dag_metadata: &Arc<BlockDAGMetadata>| {
                block_dag_metadata.all_parents_available.attach(move |_| {
                    if let Some(all_parents_available) = weak_all_parents_available.upgrade() {
                        all_parents_available.decrease();
                    }
                });
            }
        });
    }

    pub fn parents(&self) -> RwLockReadGuard<Vec<BlockMetadataRef>> {
        self.parents.read().unwrap()
    }
}
