use common::{bft::Committee, blocks::BlockMetadataRef};
use virtual_voting::{Vote, VoteBuilder};

use crate::{CommitteeSelection, Config, LeaderRotation, SlotDuration};

#[derive(Default)]
pub struct VirtualVotingParams {
    genesis_time: u64,
    committee_selection: CommitteeSelection<Config>,
    leader_rotation: LeaderRotation,
    slot_duration: SlotDuration,
    offline_threshold: u64,
}

impl Config {
    pub fn with_genesis_time(mut self, genesis_time: u64) -> Self {
        self.virtual_voting_params.genesis_time = genesis_time;
        self
    }

    pub fn with_committee_selection(
        mut self,
        committee_selection: CommitteeSelection<Self>,
    ) -> Self {
        self.virtual_voting_params.committee_selection = committee_selection;
        self
    }

    pub fn with_leader_rotation(mut self, leader_rotation: LeaderRotation) -> Self {
        self.virtual_voting_params.leader_rotation = leader_rotation;
        self
    }

    pub fn with_slot_duration(mut self, slot_duration: SlotDuration) -> Self {
        self.virtual_voting_params.slot_duration = slot_duration;
        self
    }
}

impl virtual_voting::VirtualVotingConfig for Config {
    type Source = BlockMetadataRef;

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
