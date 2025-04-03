use crate::{CommitteeSelection, Config, LeaderRotation, SlotDuration};

#[derive(Default)]
pub struct VirtualVotingParams {
    pub(crate) genesis_time: u64,
    pub(crate) committee_selection: CommitteeSelection<Config>,
    pub(crate) leader_rotation: LeaderRotation,
    pub(crate) slot_duration: SlotDuration,
    pub(crate) offline_threshold: u64,
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
