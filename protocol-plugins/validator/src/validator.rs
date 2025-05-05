use std::{
    marker::PhantomData,
    sync::{Arc, Weak},
};

use block_factory::BlockFactory;
use common::{down, extensions::ArcExt, up};
use consensus_round::ConsensusRound;
use inbox::Inbox;
use protocol::ManagedPlugin;

use crate::config::ValidatorConfig;

pub struct Validator<C: ValidatorConfig> {
    _marker: PhantomData<C>,
}

impl<C: ValidatorConfig> ManagedPlugin for Validator<C> {
    fn new(plugins: &mut protocol::Plugins) -> Arc<Self> {
        Arc::new_cyclic(|_this: &Weak<Self>| {
            let config = plugins.get::<C>().expect("Validator config not found");
            let consensus_round = plugins.load::<ConsensusRound<C>>();
            let block_factory = plugins.load::<BlockFactory<C>>();
            let inbox = plugins.load::<Inbox>();

            consensus_round.completed.subscribe(down!(config, inbox, block_factory: move |(_, new)| up!(config, inbox, block_factory: {
                let block = block_factory.create_block(&config.validator_id());
                println!("New block {:?} for round: {:?} {:?}", block.id(), new, block.parents().len());

                if let Err(e) = inbox.send(block) {
                    eprintln!("Failed to send block to inbox: {:?}", e);
                }
            }))).retain();

            Self {
                _marker: PhantomData,
            }
        })
    }
}
