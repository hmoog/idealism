use std::fmt::{Debug, Formatter, Result};

use common::bft::Committee;
use virtual_voting::{VirtualVotingConfig, Vote};

pub enum ConsensusFeedEvent<C: VirtualVotingConfig> {
    ChainIndex(Option<u64>, Option<u64>),
    HeaviestMilestoneVote(Option<Vote<C>>, Option<Vote<C>>),
    LatestAcceptedMilestone(Option<Vote<C>>, Option<Vote<C>>),
    Committee(Option<Committee>, Option<Committee>),
}

impl<C: VirtualVotingConfig> Debug for ConsensusFeedEvent<C> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            ConsensusFeedEvent::ChainIndex(old, new) => {
                write!(f, "ChainIndex({:?}, {:?})", old, new)
            }
            ConsensusFeedEvent::HeaviestMilestoneVote(old, new) => {
                write!(f, "HeaviestMilestoneVote({:?}, {:?})", old, new)
            }
            ConsensusFeedEvent::LatestAcceptedMilestone(old, new) => {
                write!(f, "LatestAcceptedMilestone({:?}, {:?})", old, new)
            }
            ConsensusFeedEvent::Committee(old, new) => {
                let old = old.as_ref().map(|x| x.commitment());
                let new = new.as_ref().map(|x| x.commitment());
                write!(f, "Committee({:?}, {:?})", old, new)
            }
        }
    }
}
