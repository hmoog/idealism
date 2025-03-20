use blockdag::BlockDAG;
use types::{
    blocks::Block,
    rx::{Event, Variable},
};
use virtual_voting::{Config, Vote};

use crate::{error::Error, events::BlocksOrderedEvent, tips::Tips};

pub struct ProtocolData<C: Config> {
    pub error: Event<Error>,
    pub blocks_ordered: Event<BlocksOrderedEvent<C>>,
    pub(crate) blocks: BlockDAG<C>,
    pub(crate) latest_accepted_milestone: Variable<Vote<C>>,
    pub(crate) tips: Tips<C>,
}

impl<C: Config> ProtocolData<C> {
    pub fn new(config: C) -> Self {
        let genesis_vote = Vote::new_genesis(config);

        let blocks = BlockDAG::new();
        let genesis_metadata = blocks.queue(Block::GenesisBlock(genesis_vote.block_id.clone()));
        genesis_metadata.vote.set(genesis_vote);

        let tips = Tips::new();
        let _ = tips.register(&genesis_metadata);

        Self {
            blocks,
            error: Event::new(),
            latest_accepted_milestone: Variable::new(),
            blocks_ordered: Event::new(),
            tips,
        }
    }
}
