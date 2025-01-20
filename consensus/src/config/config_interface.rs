use crate::committee::Committee;
use crate::committee::CommitteeMemberID;
use crate::voting::{Vote, VoteData};

pub trait ConfigInterface {
    type CommitteeMemberID: CommitteeMemberID;

    fn select_committee(&self, vote: Option<&Vote<Self>>) -> Committee<Self> where Self: Sized;

    fn leader_weight(&self, vote: &VoteData<Self>) -> u64 where Self: Sized;
}