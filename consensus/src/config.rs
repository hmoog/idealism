use crate::committee::Committee;
use crate::committee_member::CommitteeMember;
use crate::committee_member_id::CommitteeMemberID;
use crate::vote::{Vote, VoteData};

pub trait Config {
    type CommitteeMemberID: CommitteeMemberID;

    fn leader_weight(&self, vote: &VoteData<Self>) -> u64 where Self: Sized {
        vote.committee
            .member(&vote.issuer)
            .map_or(0, |member| (member.index() + vote.round) % vote.committee.size())
    }

    fn select_committee(&self, vote: Option<&Vote<Self>>) -> Committee<Self> where Self: Sized;
}

pub struct DefaultConfig;

impl Config for DefaultConfig {
    type CommitteeMemberID = u64;

    fn select_committee(&self, vote: Option<&Vote<Self>>) -> Committee<Self> where Self: Sized {
        match vote {
            Some(vote) => vote.committee().clone(),
            None => {
                Committee::from([
                    CommitteeMember::new(1),
                    CommitteeMember::new(2),
                    CommitteeMember::new(3),
                    CommitteeMember::new(4),
                ])
            },
        }
    }
}