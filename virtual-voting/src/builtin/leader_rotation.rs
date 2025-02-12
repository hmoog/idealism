use crate::{DefaultConfig, Issuer, VoteBuilder};

pub enum LeaderRotation {
    RoundRobin,
    Custom(fn(&DefaultConfig, &VoteBuilder<DefaultConfig>) -> u64),
}

impl LeaderRotation {
    pub fn dispatch(&self, config: &DefaultConfig, vote: &VoteBuilder<DefaultConfig>) -> u64 {
        match self {
            Self::RoundRobin => round_robin(vote),
            Self::Custom(strategy) => strategy(config, vote),
        }
    }
}

fn round_robin(vote: &VoteBuilder<DefaultConfig>) -> u64 {
    if let Issuer::User(issuer) = &vote.issuer {
        vote.committee.member(issuer).map_or(0, |member| {
            (member.index() + vote.round - 1) % vote.committee.size()
        })
    } else {
        0
    }
}
