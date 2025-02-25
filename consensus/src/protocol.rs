use std::sync::Arc;

use blockdag::Block as _;
use types::BlockID;
use utils::Id;
use virtual_voting::{Config, Milestone, Vote};
use zero::{Clone0, Deref0};

use crate::{
    error::Result,
    block::{Block, genesis_block},
    protocol_data::ProtocolData,
};

#[derive(Deref0, Clone0)]
pub struct Protocol<C: Config>(Arc<ProtocolData<C>>);

impl<C: Config> Protocol<C> {
    pub fn new(config: C) -> Self {
        Self(Arc::new(ProtocolData::new(config)))
    }

    pub fn run(&mut self) {
        self.blocks
            .on_ready({
                let this = self.clone();
                move |b| {
                    let _ = this.process_block(b.block()).inspect_err(|err| {
                        this.error_event.trigger(err);
                    });
                }
            })
            .forever();

        self.blocks
            .queue(Block::GenesisBlock(genesis_block::Details {
                id: BlockID::default(),
                issuer_id: Id::new(<C::IssuerID>::default()),
            }));
    }

    fn process_block(&self, block: &Block<C>) -> Result<()> {
        let vote = Vote::new(block.id().clone(), block.issuer_id(), 0, self.votes(block.parents())?)?;
        
        self.process_vote(block, vote)
    }

    fn process_vote(&self, block: &Block<C>, vote: Vote<C>) -> Result<()> {
        if let Some(milestone) = &vote.milestone {
            self.process_milestone(milestone)?;
        }

        self.votes.lock().unwrap().insert(block.id().clone(), vote);
        
        Ok(())
    }

    fn process_milestone(&self, milestone: &Milestone<C>) -> Result<()> {
        let new_milestone = Vote::try_from(&milestone.accepted)?;

        let mut guard = self.latest_accepted_milestone.get();
        *guard = Some('update: {
            if let Some(current_milestone) = guard.take() {
                if current_milestone >= new_milestone {
                    break 'update current_milestone;
                }

                self.advance_acceptance(&current_milestone, &new_milestone)?;
            }
            new_milestone
        });
        
        Ok(())
    }
    
    fn advance_acceptance(&self, old: &Vote<C>, new: &Vote<C>) -> Result<()> {
        let accepted_milestones = self.milestone_range(new, new.height()? - old.height()?)?;
        if accepted_milestones.last().expect("range must not be empty") != old {
            println!("Reorg detected");
        }
        
        for accepted_milestone in accepted_milestones.iter().rev() {
            &accepted_milestone.block_id;
        }
        
        // TODO: TRIGGER CONFIRMED BLOCKS IN ORDER

        Ok(())
    }
}
