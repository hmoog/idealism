use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use blockdag::{BlockMetadata, Error::BlockNotFound};
use common::{
    ids::BlockID,
    plugins::{Plugin, PluginRegistry},
};
use protocol::{ProtocolConfig, ProtocolPlugin, Result};

#[derive(Default)]
pub struct TipSelection<C: ProtocolConfig> {
    tips: Mutex<HashSet<BlockMetadata<C>>>,
}

impl<C: ProtocolConfig> ProtocolPlugin<C> for TipSelection<C> {
    fn process_block(&self, block: &BlockMetadata<C>) -> Result<()> {
        let metadata = block;

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
}

impl<C: ProtocolConfig> TipSelection<C> {
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

impl<C: ProtocolConfig> Plugin<dyn ProtocolPlugin<C>> for TipSelection<C> {
    fn construct(_: &mut PluginRegistry<dyn ProtocolPlugin<C>>) -> Arc<Self> {
        Arc::new(Self::default())
    }

    fn plugin(arc: Arc<Self>) -> Arc<dyn ProtocolPlugin<C>> {
        arc
    }
}
