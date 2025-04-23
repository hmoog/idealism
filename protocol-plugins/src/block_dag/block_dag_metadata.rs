use std::sync::{Arc, Mutex};

use common::{
    blocks::{BlockMetadata, BlockMetadataRef},
    rx::{Countdown, Signal},
};

pub struct BlockDAGMetadata {
    pub parents: Mutex<Vec<BlockMetadataRef>>,
    pub pending_parents: Arc<Countdown>,
    pub available: Signal<()>,
}

impl BlockDAGMetadata {
    pub fn new(parents_count: usize) -> Self {
        Self {
            parents: Mutex::new(vec![BlockMetadataRef::default(); parents_count]),
            pending_parents: Arc::new(Countdown::new(parents_count)),
            available: Signal::default(),
        }
    }

    pub fn set_parent_available(&self, index: usize, parent: &BlockMetadata) {
        self.parents.lock().unwrap()[index] = parent.downgrade();

        let pending_parents = Arc::downgrade(&self.pending_parents);

        parent
            .metadata::<Arc<BlockDAGMetadata>>()
            .subscribe(|parent_metadata| {
                parent_metadata
                    .available
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
