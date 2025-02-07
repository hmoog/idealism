use committee::{Committee, Member};

use crate::{
    CommitteeSelection, ConfigInterface, LeaderRotation, Vote, VoteBuilder,
    configuration::SlotDuration,
};

pub struct Config {
    genesis_time: u64,
    committee_selection: CommitteeSelection<Config>,
    leader_rotation: LeaderRotation,
    slot_duration: SlotDuration,
    offline_threshold: u64,
}

impl Config {
    pub fn new() -> Self {
        Self {
            genesis_time: 0,
            committee_selection: CommitteeSelection::FixedCommittee(Committee::from([
                Member::new(1),
                Member::new(2),
                Member::new(3),
                Member::new(4),
            ])),
            leader_rotation: LeaderRotation::RoundRobin,
            slot_duration: SlotDuration::Static(10),
            offline_threshold: 1,
        }
    }

    pub fn with_genesis_time(mut self, genesis_time: u64) -> Self {
        self.genesis_time = genesis_time;
        self
    }

    pub fn with_committee_selection(
        mut self,
        committee_selection: CommitteeSelection<Self>,
    ) -> Self {
        self.committee_selection = committee_selection;
        self
    }

    pub fn with_leader_rotation(mut self, leader_rotation: LeaderRotation) -> Self {
        self.leader_rotation = leader_rotation;
        self
    }

    pub fn with_slot_duration(mut self, slot_duration: SlotDuration) -> Self {
        self.slot_duration = slot_duration;
        self
    }
}

impl ConfigInterface for Config {
    type IssuerID = u64;

    fn genesis_time(&self) -> u64 {
        self.genesis_time
    }

    fn slot_oracle(&self, vote: &VoteBuilder<Self>) -> u64 {
        self.slot_duration.map_slot(self, vote)
    }

    fn offline_threshold(&self) -> u64 {
        self.offline_threshold
    }

    fn select_committee(&self, vote: Option<&Vote<Self>>) -> Committee<Self::IssuerID>
    where
        Self: Sized,
    {
        self.committee_selection.dispatch(self, vote)
    }

    fn leader_weight(&self, vote: &VoteBuilder<Self>) -> u64
    where
        Self: Sized,
    {
        self.leader_rotation.dispatch(self, vote)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}
