use crate::bft_committee::Committee;
use crate::bft_committee::CommitteeMemberID;
use crate::{Vote, VoteData};

pub trait ConfigInterface {
    type CommitteeMemberID: CommitteeMemberID;

    fn select_committee(&self, vote: Option<&Vote<Self>>) -> Committee<Self> where Self: Sized;

    fn leader_weight(&self, vote: &VoteData<Self>) -> u64 where Self: Sized;
}