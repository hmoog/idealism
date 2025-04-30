use std::sync::{Arc, Mutex};

use block_dag::BlockDAG;
use common::{
    bft::Committee,
    blocks::BlockMetadata,
    errors::Error::BlockNotFound,
    rx::{
        Callbacks, Event, Subscription, UpdateType,
        UpdateType::{Notify, Retain},
        Variable,
    },
};
use protocol::{ManagedPlugin, Plugins};
use virtual_voting::{VirtualVotingConfig, Vote};

use crate::{AcceptanceState, AcceptedBlocks, ConsensusMetadata};

#[derive(Default)]
pub struct Consensus<C: VirtualVotingConfig> {
    pub chain_index: Variable<u64>,
    pub heaviest_milestone_vote: Variable<Vote<C>>,
    pub latest_accepted_milestone: Variable<Vote<C>>,
    pub committee: Variable<Committee>,
    pub accepted_blocks: Event<AcceptedBlocks>,
    block_dag_subscription: Mutex<Option<Subscription<Callbacks<BlockMetadata>>>>,
}

impl<C: VirtualVotingConfig> Consensus<C> {
    fn setup(self: Arc<Self>, plugins: &mut Plugins) -> Arc<Self> {
        let weak_consensus = Arc::downgrade(&self);

        *self.block_dag_subscription.lock().expect("failed to lock") =
            Some(plugins.load::<BlockDAG>().subscribe(move |block| {
                let weak_consensus = weak_consensus.clone();

                block
                    .metadata::<Arc<Vote<C>>>()
                    .subscribe(move |vote: &Arc<Vote<C>>| {
                        if let Some(consensus) = weak_consensus.upgrade() {
                            if let Err(err) = consensus.process_vote(vote) {
                                // TODO: handle the error more elegantly
                                println!("{:?}", err);
                            }
                        }
                    })
                    .retain();
            }));

        self
    }

    fn shutdown(&self) {
        self.block_dag_subscription.lock().unwrap().take();
    }

    fn process_vote(&self, vote: &Vote<C>) -> virtual_voting::Result<()> {
        if vote.milestone.is_some() {
            self.update_heaviest_milestone_vote(vote)?;
            self.update_latest_accepted_milestone(vote)?;
        };

        Ok(())
    }

    fn update_heaviest_milestone_vote(&self, vote: &Vote<C>) -> virtual_voting::Result<()> {
        self.heaviest_milestone_vote.compute(|old| {
            let result = match old {
                Some(old) if old >= *vote => Retain(Some(old)),
                _ => Notify(old, Some(vote.clone())),
            };

            if let Notify(_, Some(new)) = &result {
                self.committee
                    .set_if_none_or(new.committee.clone(), |old, new| {
                        new.commitment() != old.commitment()
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
                match milestones.last().expect("must exist") == old {
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
            let block = accepted_milestone.source.upgrade().ok_or(BlockNotFound)?;
            let past_cone = BlockDAG::past_cone(&block, |b| {
                Ok(!b.try_get::<Arc<ConsensusMetadata>>()?.is_accepted(0))
            })?;

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

impl<C: VirtualVotingConfig> ManagedPlugin for Consensus<C> {
    fn construct(plugins: &mut Plugins) -> Arc<Self> {
        Arc::new(Self::default()).setup(plugins)
    }

    fn shutdown(&self) {
        self.shutdown();
    }
}
