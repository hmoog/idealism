use std::{collections::HashSet, sync::Mutex};

use blockdag::BlockMetadata;
use types::{ids::BlockID, rx::Variable};
use virtual_voting::Config;

use crate::error::{Error, Result};

pub struct Tips<C: Config> {
    tips: Mutex<HashSet<BlockMetadata<C>>>,
    _heaviest: Variable<BlockMetadata<C>>,
}

impl<C: Config> Tips<C> {
    pub fn new() -> Self {
        Self {
            tips: Mutex::new(HashSet::new()),
            _heaviest: Variable::new(),
        }
    }

    pub fn register(&self, metadata: &BlockMetadata<C>) -> Result<Vec<BlockMetadata<C>>> {
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

                    return Err(Error::BlockNotFound);
                }
            }
        }
        tips.insert(metadata.clone());

        Ok(removed_tips)
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
