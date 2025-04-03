use blockdag::BlockMetadataRef;
use common::bft::Committee;
use virtual_voting::{Vote, VoteBuilder};

use crate::Config;

impl virtual_voting::VirtualVotingConfig for Config {
    type Source = BlockMetadataRef<Self>;

    fn genesis_time(&self) -> u64 {
        self.virtual_voting_params.genesis_time
    }

    fn slot_oracle(&self, time: u64) -> u64 {
        self.virtual_voting_params
            .slot_duration
            .map_slot(self, time)
    }

    fn offline_threshold(&self) -> u64 {
        self.virtual_voting_params.offline_threshold
    }

    fn select_committee(&self, vote: Option<&Vote<Self>>) -> Committee
    where
        Self: Sized,
    {
        self.virtual_voting_params
            .committee_selection
            .dispatch(self, vote)
    }

    fn leader_weight(&self, vote: &VoteBuilder<Self>) -> u64 {
        self.virtual_voting_params
            .leader_rotation
            .dispatch(self, vote)
    }
}
