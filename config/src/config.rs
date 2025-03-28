use blockdag::BlockMetadataRef;
use types::{
    bft::{Committee, Member},
    ids::{BlockID, IssuerID},
};
use virtual_voting::{Vote, VoteBuilder};

use crate::{CommitteeSelection, LeaderRotation, SlotDuration};

pub struct Config {
    genesis_time: u64,
    committee_selection: CommitteeSelection<Config>,
    leader_rotation: LeaderRotation,
    slot_duration: SlotDuration,
    offline_threshold: u64,
}

impl Config {
    pub fn new() -> Self {
        let issuer_id_1 = IssuerID::from([1u8; 32]);
        let issuer_id_2 = IssuerID::from([2u8; 32]);
        let issuer_id_3 = IssuerID::from([3u8; 32]);
        let issuer_id_4 = IssuerID::from([4u8; 32]);

        Self {
            genesis_time: 0,
            committee_selection: CommitteeSelection::FixedCommittee(Committee::from([
                Member::new(issuer_id_1),
                Member::new(issuer_id_2),
                Member::new(issuer_id_3),
                Member::new(issuer_id_4),
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

impl blockdag::BlockDAGConfig for Config {
    type ErrorType = protocol::Error;

    fn genesis_block_id(&self) -> BlockID {
        BlockID::default()
    }
}

impl protocol::ProtocolConfig for Config {}

impl virtual_voting::VirtualVotingConfig for Config {
    type Source = BlockMetadataRef<Self>;

    fn genesis_time(&self) -> u64 {
        self.genesis_time
    }

    fn slot_oracle(&self, time: u64) -> u64 {
        self.slot_duration.map_slot(self, time)
    }

    fn offline_threshold(&self) -> u64 {
        self.offline_threshold
    }

    fn select_committee(&self, vote: Option<&Vote<Self>>) -> Committee
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
