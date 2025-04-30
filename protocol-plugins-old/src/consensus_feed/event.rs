use common::{bft::Committee, blocks::BlockMetadataRef};
use virtual_voting::{VirtualVotingConfig, Vote};

pub enum ConsensusFeedEvent<C: VirtualVotingConfig<Source = BlockMetadataRef>> {
    ChainIndexUpdated(Option<u64>, Option<u64>),
    HeaviestMilestoneVoteUpdated(Option<Vote<C>>, Option<Vote<C>>),
    LatestAcceptedMilestoneUpdated(Option<Vote<C>>, Option<Vote<C>>),
    CommitteeUpdated(Option<Committee>, Option<Committee>),
}

impl<C: VirtualVotingConfig<Source = BlockMetadataRef>> std::fmt::Debug for ConsensusFeedEvent<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConsensusFeedEvent::ChainIndexUpdated(old, new) => f
                .debug_struct("ChainIndexUpdated")
                .field("old", old)
                .field("new", new)
                .finish(),
            ConsensusFeedEvent::HeaviestMilestoneVoteUpdated(old, new) => f
                .debug_struct("HeaviestMilestoneVoteUpdated")
                .field("old", old)
                .field("new", new)
                .finish(),
            ConsensusFeedEvent::LatestAcceptedMilestoneUpdated(old, new) => f
                .debug_struct("LatestAcceptedMilestoneUpdated")
                .field("old", old)
                .field("new", new)
                .finish(),
            ConsensusFeedEvent::CommitteeUpdated(old, new) => f
                .debug_struct("CommitteeUpdated")
                .field("old", &old.as_ref().map(|x| x.commitment()))
                .field("new", &new.as_ref().map(|x| x.commitment()))
                .finish(),
        }
    }
}
