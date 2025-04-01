use std::fmt::Debug;

use blockdag::BlockMetadata;
use indexmap::IndexSet;
use protocol::ProtocolConfig;

pub struct AcceptedBlocks<C: ProtocolConfig> {
    pub height: u64,
    pub rounds: Vec<IndexSet<BlockMetadata<C>>>,
}

impl<C: ProtocolConfig> Debug for AcceptedBlocks<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlocksOrderedEvent")
            .field("current_height", &self.height)
            .field("ordered_blocks", &self.rounds)
            .finish()
    }
}
