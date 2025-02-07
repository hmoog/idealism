use crate::{Committee, CommitteeMemberID, Vote, VoteBuilder};

pub trait ConfigInterface: Default {
    type IssuerID: CommitteeMemberID;

    fn genesis_time(&self) -> u64;

    fn slot_oracle(&self, vote: &VoteBuilder<Self>) -> u64;

    fn offline_threshold(&self) -> u64;

    fn select_committee(&self, vote: Option<&Vote<Self>>) -> Committee<Self>
    where
        Self: Sized;

    fn leader_weight(&self, vote: &VoteBuilder<Self>) -> u64
    where
        Self: Sized;
}
