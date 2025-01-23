use crate::{Committee, CommitteeMemberID, Vote, VoteData};

pub trait ConfigInterface: Default {
    type CommitteeMemberID: CommitteeMemberID;

    fn select_committee(&self, vote: Option<&Vote<Self>>) -> Committee<Self> where Self: Sized;

    fn leader_weight(&self, vote: &VoteData<Self>) -> u64 where Self: Sized;
}