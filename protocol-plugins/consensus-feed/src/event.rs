use common::bft::Committee;
use virtual_voting::{VirtualVotingConfig, Vote};

pub enum ConsensusFeedEvent<C: VirtualVotingConfig> {
    ChainIndex(Option<u64>, Option<u64>),
    HeaviestMilestoneVote(Option<Vote<C>>, Option<Vote<C>>),
    LatestAcceptedMilestone(Option<Vote<C>>, Option<Vote<C>>),
    Committee(Option<Committee>, Option<Committee>),
}

impl<C: VirtualVotingConfig> std::fmt::Debug for ConsensusFeedEvent<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsensusFeedEvent::ChainIndex(old, new) => f
                .debug_struct("ChainIndexUpdated")
                .field("old", old)
                .field("new", new)
                .finish(),
            ConsensusFeedEvent::HeaviestMilestoneVote(old, new) => f
                .debug_struct("HeaviestMilestoneVoteUpdated")
                .field("old", old)
                .field("new", new)
                .finish(),
            ConsensusFeedEvent::LatestAcceptedMilestone(old, new) => f
                .debug_struct("LatestAcceptedMilestoneUpdated")
                .field("old", old)
                .field("new", new)
                .finish(),
            ConsensusFeedEvent::Committee(old, new) => f
                .debug_struct("CommitteeUpdated")
                .field("old", &old.as_ref().map(|x| x.commitment()))
                .field("new", &new.as_ref().map(|x| x.commitment()))
                .finish(),
        }
    }
}
