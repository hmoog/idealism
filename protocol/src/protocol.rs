use std::sync::Arc;

use blockdag::{Accepted, BlockDAG, BlockMetadata, Error::BlockNotFound};
use indexmap::IndexSet;
use types::{
    bft::Committee,
    blocks::{Block, NetworkBlock},
    ids::IssuerID,
    rx::{
        ResourceGuard, UpdateType,
        UpdateType::{Notify, Retain},
        Variable,
    },
};
use virtual_voting::{Config, Vote};
use zero::{Clone0, Deref0};

use crate::{
    Events,
    error::{Error, Result},
    events::BlocksOrdered,
    tips::Tips,
};

#[derive(Deref0, Clone0, Default)]
pub struct Protocol<C: Config>(Arc<Data<C>>);

#[derive(Default)]
pub struct Data<C: Config> {
    pub events: Events<C>,
    pub block_dag: BlockDAG<C>,
    pub state: State<C>,
    pub tips: Tips<C>,
}

#[derive(Default)]
pub struct State<C: Config> {
    pub chain_index: Variable<u64>,
    pub round: Arc<Variable<u64>>,
    pub committee: Arc<Variable<Committee>>,
    pub heaviest_milestone: Variable<Vote<C>>,
    pub latest_accepted_milestone: Variable<Vote<C>>,
}

impl<C: Config> Protocol<C> {
    pub fn new(config: C) -> Self {
        let protocol = Self::default();
        protocol.init(Vote::new_genesis(config));
        protocol.start();
        protocol
    }

    pub fn new_block(&self, issuer: &IssuerID) -> Block {
        Block::from(NetworkBlock {
            parents: self.tips.get(),
            issuer_id: issuer.clone(),
        })
    }

    fn init(&self, genesis: Vote<C>) {
        let genesis_block = Block::GenesisBlock(genesis.block_id.clone());
        let genesis_metadata = self.block_dag.attach(genesis_block);
        genesis_metadata.vote.set(genesis);

        self.tips.register(&genesis_metadata).expect("must succeed");
    }

    fn start(&self) {
        // derive state from block_dag
        self.block_dag
            .on_block_ready({
                let protocol = self.clone();

                move |block_metadata| {
                    if let Err(err) = protocol.process_block(block_metadata) {
                        protocol.events.error.trigger(&err);
                    }
                }
            })
            .forever();

        // derive committee and round from the heaviest milestone
        self.state
            .heaviest_milestone
            .subscribe({
                let round = self.state.round.clone();
                let committee = self.state.committee.clone();

                move |update| {
                    if let Some(heaviest_milestone) = &update.1 {
                        round.track_max(heaviest_milestone.round);

                        committee
                            .set_if_none_or(heaviest_milestone.committee.clone(), |old, new| {
                                new.commitment() != old.commitment()
                            });
                    }
                }
            })
            .forever();
    }

    fn process_block(&self, metadata: &ResourceGuard<BlockMetadata<C>>) -> Result<()> {
        match &metadata.block {
            Block::NetworkBlock(id, network_block) => {
                // reconstruct vote
                let vote = Vote::new(
                    id.clone(),
                    &network_block.issuer_id,
                    0,
                    metadata.referenced_votes()?,
                )?;

                if let Some(milestone) = &vote.milestone {
                    // advance acceptance
                    let new_accepted = Vote::try_from(&milestone.accepted)?;
                    self.state
                        .latest_accepted_milestone
                        .compute::<Error, _>(|old| match old {
                            Some(old_accepted) if old_accepted >= new_accepted => {
                                Retain(Some(old_accepted))
                            }
                            Some(old_accepted) => {
                                match self.advance_acceptance(&old_accepted, &new_accepted) {
                                    Err(err) => UpdateType::Error(Some(old_accepted), err),
                                    _ => Notify(Some(old_accepted), Some(new_accepted)),
                                }
                            }
                            _ => Notify(old, Some(new_accepted)),
                        })?;

                    // track the heaviest milestone
                    self.state.heaviest_milestone.track_max(vote.clone());
                }

                self.tips.register(metadata)?;

                metadata.vote.set(vote);

                Ok(())
            }
            _ => Err(Error::UnsupportedBlockType),
        }
    }

    fn advance_acceptance(&self, old: &Vote<C>, new: &Vote<C>) -> Result<()> {
        let current_height = old.height()?;
        let new_height = new.height()?;

        match new_height.checked_sub(current_height) {
            None | Some(0) => panic!("TODO: implement reorg"),
            Some(accepted_height) => {
                let accepted_milestones = new.milestone_range(accepted_height)?;

                match accepted_milestones.last().expect("must exist") == old {
                    false => panic!("TODO: implement reorg"),
                    true => {
                        let ordered_blocks =
                            self.ordered_blocks(current_height, accepted_milestones)?;

                        self.events.blocks_ordered.trigger(&BlocksOrdered {
                            current_height,
                            ordered_blocks,
                        })
                    }
                }
            }
        }

        Ok(())
    }

    fn ordered_blocks(
        &self,
        current_height: u64,
        accepted_milestones: Vec<Vote<C>>,
    ) -> Result<Vec<IndexSet<BlockMetadata<C>>>> {
        let mut ordered_blocks = Vec::with_capacity(accepted_milestones.len());

        for (height_index, accepted_milestone) in accepted_milestones.iter().rev().enumerate() {
            let milestone_block = self
                .block_dag
                .get(&accepted_milestone.block_id)
                .ok_or(Error::BlockDagErr(BlockNotFound))?;
            let past_cone = milestone_block.past_cone(|b| !b.is_accepted(0))?;

            for (round_index, block) in past_cone.iter().rev().enumerate() {
                block.accepted.set(Accepted {
                    chain_id: 0,
                    height: current_height + (height_index + 1) as u64,
                    round_index: round_index as u64,
                });
            }

            ordered_blocks.push(past_cone);
        }

        Ok(ordered_blocks)
    }
}
