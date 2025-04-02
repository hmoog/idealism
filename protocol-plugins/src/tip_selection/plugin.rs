use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};

use blockdag::{
    BlockMetadata,
    Error::{BlockNotFound, VoteNotFound},
};
use common::{
    ids::BlockID,
    plugins::{Plugin, PluginManager},
};
use protocol::{Protocol, ProtocolConfig, ProtocolPlugin, Result};
use virtual_voting::Vote;

#[derive(Default)]
pub struct TipSelection<C: ProtocolConfig> {
    tips: Mutex<HashSet<BlockMetadata<C>>>,
}

impl<C: ProtocolConfig> ProtocolPlugin<C> for TipSelection<C> {
    fn process_vote(&self, _protocol: &Protocol<C>, vote: &Vote<C>) -> Result<()> {
        let metadata = vote.source.upgrade().ok_or(VoteNotFound)?;

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
    fn construct(_: &mut PluginManager<dyn ProtocolPlugin<C>>) -> Arc<Self> {
        Arc::new(Self::default())
    }

    fn plugin(arc: Arc<Self>) -> Arc<dyn ProtocolPlugin<C>> {
        arc
    }
}
