use common::rx::Signal;

use crate::AcceptanceState;

#[derive(Default)]
pub struct ConsensusMetadata {
    pub accepted: Signal<AcceptanceState>,
}

impl ConsensusMetadata {
    pub fn is_accepted(&self, chain_id: u64) -> bool {
        self.accepted
            .get()
            .as_ref()
            .is_some_and(|a| a.chain_id == chain_id)
    }
}
