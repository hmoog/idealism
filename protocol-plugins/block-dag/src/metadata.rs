use std::sync::{Arc, Mutex};

use common::{
    blocks::{BlockMetadata, BlockMetadataRef},
    rx::Countdown,
};

pub struct BlockDAGMetadata {
    pub parents: Mutex<Vec<BlockMetadataRef>>,
    pub all_parents_available: Arc<Countdown>,
}

impl BlockDAGMetadata {
    pub fn new(parents_count: usize) -> Self {
        Self {
            parents: Mutex::new(vec![BlockMetadataRef::default(); parents_count]),
            all_parents_available: Arc::new(Countdown::new(parents_count)),
        }
    }

    pub fn set_parent_available(self: &Arc<Self>, index: usize, parent: &BlockMetadata) {
        self.parents.lock().unwrap()[index] = parent.downgrade();

        let pending_parents = Arc::downgrade(&self.all_parents_available);

        parent
            .metadata::<Arc<BlockDAGMetadata>>()
            .subscribe(|parent_metadata| {
                parent_metadata
                    .all_parents_available
                    .subscribe(move |_| {
                        if let Some(pending_parents) = pending_parents.upgrade() {
                            pending_parents.decrease();
                        }
                    })
                    .retain();
            })
            .retain();
    }
}
