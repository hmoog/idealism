use std::{collections::HashSet, sync::Mutex};

use blockdag::{BlockMetadata, Error::BlockNotFound};
use common::ids::BlockID;

use crate::{Protocol, ProtocolConfig, Result};

#[derive(Default)]
pub struct Tips<C: ProtocolConfig> {
    tips: Mutex<HashSet<BlockMetadata<C>>>,
}

impl<C: ProtocolConfig> Tips<C> {
    pub fn init(&self, protocol: &Protocol<C>) {
        let mut tips = self.tips.lock().expect("failed to lock");
        tips.insert(protocol.block_dag.genesis().clone());
    }

    pub fn process_vote(&self, metadata: &BlockMetadata<C>) -> Result<()> {
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
