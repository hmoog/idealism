use std::{
    marker::PhantomData,
    sync::{Arc, Weak},
};

use block_factory::BlockFactory;
use common::{down, extensions::ArcExt, up, with};
use consensus_round::ConsensusRound;
use inbox::Inbox;
use protocol::ManagedPlugin;
use tracing::{Span, error, info, info_span};

use crate::config::ValidatorConfig;

pub struct Validator<C: ValidatorConfig> {
    span: Span,
    _marker: PhantomData<C>,
}

impl<C: ValidatorConfig> ManagedPlugin for Validator<C> {
    fn new(plugins: &mut protocol::Plugins) -> Arc<Self> {
        Arc::new_cyclic(|this: &Weak<Self>| {
            let config = plugins.get::<C>().expect("Validator config not found");
            let consensus_round = plugins.load::<ConsensusRound<C>>();
            let block_factory = plugins.load::<BlockFactory<C>>();
            let inbox = plugins.load::<Inbox>();

            consensus_round.completed.subscribe(with!(this: down!(config, inbox, block_factory: move |(_, new)| up!(this, config, inbox, block_factory: {
                this.span.in_scope(|| {
                    let block = block_factory.create_block(&config.validator_id());
                    info!("issuing block for round {:?} (id={:?})", new.unwrap_or(0), block.id());
                    if let Err(e) = inbox.send(block) {
                        error!("issuing block for round {:?} failed: {e}", new);
                    }
                })
            })))).retain();

            Self {
                span: info_span!("validator"),
                _marker: PhantomData,
            }
        })
    }

    fn span(&self) -> Span {
        self.span.clone()
    }
}
