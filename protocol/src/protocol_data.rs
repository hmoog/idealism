use blockdag::BlockDAG;
use types::{blocks::Block, rx::Variable};
use virtual_voting::{Config, Vote};

use crate::{State, events::Events, tips::Tips};

pub struct ProtocolData<C: Config> {
    pub events: Events<C>,
    pub block_dag: BlockDAG<C>,
    pub state: State<C>,
    pub(crate) latest_accepted_milestone: Variable<Vote<C>>,
    pub(crate) tips: Tips<C>,
}

impl<C: Config> ProtocolData<C> {
    pub fn new(config: C) -> Self {
        let protocol_data = Self {
            events: Events::new(),
            block_dag: BlockDAG::new(),
            state: State::new(),
            latest_accepted_milestone: Variable::new(),
            tips: Tips::new(),
        };

        let genesis_vote = Vote::new_genesis(config);
        let genesis_metadata = protocol_data
            .block_dag
            .attach(Block::GenesisBlock(genesis_vote.block_id.clone()));

        genesis_metadata.vote.set(genesis_vote);

        let _ = protocol_data.tips.register(&genesis_metadata);

        protocol_data
    }
}
