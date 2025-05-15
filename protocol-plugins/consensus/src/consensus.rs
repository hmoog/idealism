use std::sync::{Arc, Mutex, Weak};

use async_trait::async_trait;
use block_dag::{BlockDAG, BlockMetadataExt};
use common::{
    bft::Committee,
    blocks::BlockMetadata,
    down,
    rx::{
        Callbacks, Event, Subscription, UpdateType,
        UpdateType::{Notify, Retain},
        Variable,
    },
    up, with,
};
use protocol::{ManagedPlugin, Plugins};
use tracing::{Span, error, info, info_span, trace};
use virtual_voting::{VirtualVotingConfig, Vote};

use crate::{AcceptanceState, AcceptedBlocks, ConsensusMetadata};

pub struct Consensus<C: VirtualVotingConfig> {
    pub chain_index: Variable<u64>,
    pub heaviest_milestone_vote: Variable<Vote<C>>,
    pub latest_accepted_milestone: Variable<Vote<C>>,
    pub committee: Variable<Committee>,
    pub accepted_blocks: Event<AcceptedBlocks>,
    block_dag_subscription: Mutex<Option<Subscription<Callbacks<BlockMetadata>>>>,
    span: Span,
}

#[async_trait]
impl<C: VirtualVotingConfig> ManagedPlugin for Consensus<C> {
    fn new(plugins: &mut Plugins) -> Arc<Self> {
        Arc::new_cyclic(|this: &Weak<Self>| {
            let block_dag = plugins.load::<BlockDAG>();

            Self {
                chain_index: Default::default(),
                heaviest_milestone_vote: Default::default(),
                latest_accepted_milestone: Default::default(),
                committee: Default::default(),
                accepted_blocks: Default::default(),
                block_dag_subscription: Mutex::new(Some(block_dag.block_available.subscribe(
                    with!(this: move |block| {
                        block.attach(down!(block: with!(this: move |vote| up!(this, block: {
                            block.set(Arc::new(ConsensusMetadata::default()));

                            this.process_vote(vote).unwrap_or_else(|e| error!("{:?}", e))
                        }))))
                    }),
                ))),
                span: info_span!("consensus"),
            }
        })
    }

    async fn shutdown(&self) {
        self.block_dag_subscription.lock().unwrap().take();
    }

    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl<C: VirtualVotingConfig> Consensus<C> {
    fn process_vote(&self, vote: &Vote<C>) -> virtual_voting::Result<()> {
        if vote.milestone.is_some() {
            self.update_heaviest_milestone_vote(vote)?;
            self.update_latest_accepted_milestone(vote)?;
        }

        Ok(())
    }

    fn update_heaviest_milestone_vote(&self, vote: &Vote<C>) -> virtual_voting::Result<()> {
        self.heaviest_milestone_vote.compute(|old| {
            let result = match old {
                Some(old) if old >= *vote => Retain(Some(old)),
                _ => Notify(old, Some(vote.clone())),
            };

            if let Notify(_, Some(new)) = &result {
                trace!("heaviest milestone vote updated: {:?}", new);
                let _ = self
                    .committee
                    .compute::<(), _>(move |current| match current {
                        Some(old) if new.committee.commitment() == old.commitment() => {
                            Retain(Some(old))
                        }
                        _ => {
                            info!("committee updated: {:?}", new.committee.commitment());
                            Notify(current, Some(new.committee.clone()))
                        }
                    });
            }

            result
        })
    }

    fn update_latest_accepted_milestone(&self, vote: &Vote<C>) -> virtual_voting::Result<()> {
        let new = Vote::try_from(&vote.milestone()?.accepted)?;

        self.latest_accepted_milestone.compute(|old| match old {
            Some(old) if old >= new => Retain(Some(old)),
            Some(old) => match self.advance_acceptance(&old, &new) {
                Err(err) => UpdateType::Error(Some(old), err),
                _ => Notify(Some(old), Some(new)),
            },
            _ => Notify(old, Some(new)),
        })
    }

    fn advance_acceptance(&self, old: &Vote<C>, new: &Vote<C>) -> virtual_voting::Result<()> {
        let height = old.height()?;
        match new.height()?.checked_sub(height) {
            None | Some(0) => panic!("TODO: implement reorg"),
            Some(range) => {
                let milestones = new.milestone_range(range)?;
                let last_milestone =
                    Vote::try_from(milestones.last().expect("must exist").prev_milestone()?)?;
                match &last_milestone == old {
                    false => panic!("TODO: implement reorg"),
                    true => self
                        .accepted_blocks
                        .trigger(&self.accepted_blocks(height, milestones)?),
                }
            }
        }

        Ok(())
    }

    fn accepted_blocks(
        &self,
        height: u64,
        milestones: Vec<Vote<C>>,
    ) -> virtual_voting::Result<AcceptedBlocks> {
        let mut accepted_blocks = AcceptedBlocks {
            height,
            rounds: Vec::with_capacity(milestones.len()),
        };

        for (height_index, accepted_milestone) in milestones.iter().rev().enumerate() {
            let block = accepted_milestone.source.try_upgrade()?;
            let past_cone =
                block.past_cone(|b| Ok(!b.try_get::<Arc<ConsensusMetadata>>()?.is_accepted(0)))?;

            for (round_index, block) in past_cone.iter().rev().enumerate() {
                block
                    .try_get::<Arc<ConsensusMetadata>>()?
                    .accepted
                    .set(AcceptanceState {
                        chain_id: 0,
                        height: height + (height_index + 1) as u64,
                        round_index: round_index as u64,
                    });
            }

            accepted_blocks.rounds.push(past_cone);
        }

        Ok(accepted_blocks)
    }
}
