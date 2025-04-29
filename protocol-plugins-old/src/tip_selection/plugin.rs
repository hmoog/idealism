use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use block_dag::BlockDAGMetadata;
use common::{
    blocks::BlockMetadata,
    errors::{Error::BlockNotFound, Result},
    ids::BlockID,
    plugins::{Plugin, PluginRegistry},
};
use protocol::{ProtocolConfig, ProtocolPlugin};

#[derive(Default)]
pub struct TipSelection {
    tips: Mutex<HashSet<BlockMetadata>>,
}

impl<C: ProtocolConfig> ProtocolPlugin<C> for TipSelection {
    fn shutdown(&self) {
        todo!()
    }
}

impl TipSelection {
    fn process_block(&self, block: &BlockMetadata) -> Result<()> {
        let metadata = block;

        let block_dag_metadata = metadata.try_get::<Arc<BlockDAGMetadata>>()?;
        let locked_parents = block_dag_metadata.parents.lock().unwrap();
        let parent_refs = locked_parents.iter();
        let mut removed_tips = Vec::with_capacity(block.block.parents().len());
        let mut tips = self.tips.lock().expect("failed to lock");
        for parent_ref in parent_refs {
            match parent_ref.upgrade() {
                Some(parent) => {
                    if tips.remove(&parent) {
                        removed_tips.push(parent);
                    }
                }
                None => {
                    tips.extend(removed_tips.into_iter());

                    return Err(BlockNotFound);
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

impl<C: ProtocolConfig> Plugin<dyn ProtocolPlugin<C>> for TipSelection {
    fn construct(_: &mut PluginRegistry<dyn ProtocolPlugin<C>>) -> Arc<Self> {
        Arc::new(Self::default())
    }

    fn plugin(arc: Arc<Self>) -> Arc<dyn ProtocolPlugin<C>> {
        arc
    }
}
