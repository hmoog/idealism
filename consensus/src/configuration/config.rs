use crate::{
    Committee, CommitteeMember, CommitteeSelection, ConfigInterface, LeaderRotation, Vote, VoteData,
};

pub struct Config {
    committee_selection: CommitteeSelection,
    leader_rotation: LeaderRotation,
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_committee_selection(mut self, committee_selection: CommitteeSelection) -> Self {
        self.committee_selection = committee_selection;
        self
    }

    pub fn with_leader_rotation(mut self, leader_rotation: LeaderRotation) -> Self {
        self.leader_rotation = leader_rotation;
        self
    }
}

impl ConfigInterface for Config {
    type CommitteeMemberID = u64;

    fn select_committee(&self, vote: Option<&Vote<Self>>) -> Committee<Self>
    where
        Self: Sized,
    {
        self.committee_selection.dispatch(self, vote)
    }

    fn leader_weight(&self, vote: &VoteData<Self>) -> u64
    where
        Self: Sized,
    {
        self.leader_rotation.dispatch(self, vote)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            committee_selection: CommitteeSelection::FixedCommittee(Committee::from([
                CommitteeMember::new(1),
                CommitteeMember::new(2),
                CommitteeMember::new(3),
                CommitteeMember::new(4),
            ])),
            leader_rotation: LeaderRotation::RoundRobin,
        }
    }
}
