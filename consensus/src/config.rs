use std::hash::Hash;
use crate::committee::Committee;
use crate::vote::Vote;

pub trait Config {
    type CommitteeMemberID: PartialEq + Eq + Hash + Default;

    fn select_committee(&self, vote: &Vote<Self::CommitteeMemberID>) -> Committee<Self::CommitteeMemberID>;
}

pub struct DefaultConfig;

impl Config for DefaultConfig {
    type CommitteeMemberID = u64;

    fn select_committee(&self, _vote: &Vote<Self::CommitteeMemberID>) -> Committee<Self::CommitteeMemberID> {
        unimplemented!()
    }
}