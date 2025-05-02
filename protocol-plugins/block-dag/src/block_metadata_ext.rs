use std::{collections::VecDeque, sync::Arc};

use common::{blocks::BlockMetadata, errors::Result};
use indexmap::IndexSet;

use crate::BlockDAGMetadata;

pub trait BlockDAGBlockMetadataExt {
    fn past_cone<F: Fn(&BlockMetadata) -> Result<bool>>(
        &self,
        should_visit: F,
    ) -> Result<IndexSet<BlockMetadata>>;
}

impl BlockDAGBlockMetadataExt for BlockMetadata {
    fn past_cone<F: Fn(&BlockMetadata) -> Result<bool>>(
        &self,
        should_visit: F,
    ) -> Result<IndexSet<BlockMetadata>> {
        let mut past_cone = IndexSet::new();

        if should_visit(self)? && past_cone.insert(self.clone()) {
            let mut queue = VecDeque::from([self.clone()]);

            while let Some(current) = queue.pop_front() {
                for parent_ref in current
                    .try_get::<Arc<BlockDAGMetadata>>()?
                    .parents
                    .lock()
                    .unwrap()
                    .iter()
                {
                    let parent_block = parent_ref.try_upgrade()?;

                    if should_visit(&parent_block)? && past_cone.insert(parent_block.clone()) {
                        queue.push_back(parent_block);
                    }
                }
            }
        }

        Ok(past_cone)
    }
}
