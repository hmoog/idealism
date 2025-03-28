use std::{collections::HashSet, sync::Mutex};

use blockdag::{BlockMetadata, Error::BlockNotFound};
use types::ids::BlockID;

use crate::{ProtocolConfig, Result};

#[derive(Default)]
pub struct Tips<C: ProtocolConfig> {
    tips: Mutex<HashSet<BlockMetadata<C>>>,
}

impl<C: ProtocolConfig> Tips<C> {
    pub fn init(&self, genesis: BlockMetadata<C>) {
        let mut tips = self.tips.lock().expect("failed to lock");
        tips.insert(genesis);
    }

    pub fn apply(&self, metadata: &BlockMetadata<C>) -> Result<()> {
        let parent_refs = metadata.parents();
        let mut removed_tips = Vec::with_capacity(parent_refs.len());

        let mut tips = self.tips.lock().expect("failed to lock");
        for parent_ref in parent_refs.iter() {
            match parent_ref.upgrade() {
                Some(parent) => {
                    if tips.remove(&parent) {
                        removed_tips.push(parent);
                    }
                }
                None => {
                    tips.extend(removed_tips.into_iter());

                    return Err(BlockNotFound.into());
                }
            }
        }
        tips.insert(metadata.clone());

        Ok(())
    }

    pub fn get(&self) -> Vec<BlockID> {
        self.tips
            .lock()
            .expect("failed to lock")
            .iter()
            .map(|x| x.block.id())
            .cloned()
            .collect()
    }
}
