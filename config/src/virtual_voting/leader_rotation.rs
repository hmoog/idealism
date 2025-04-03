use virtual_voting::{Issuer, VoteBuilder};

use crate::Config;

pub enum LeaderRotation {
    RoundRobin,
    Custom(fn(&Config, &VoteBuilder<Config>) -> u64),
}

impl LeaderRotation {
    pub fn dispatch(&self, config: &Config, vote: &VoteBuilder<Config>) -> u64 {
        match self {
            Self::RoundRobin => round_robin(vote),
            Self::Custom(strategy) => strategy(config, vote),
        }
    }
}

fn round_robin(vote: &VoteBuilder<Config>) -> u64 {
    if let Issuer::User(issuer) = &vote.issuer {
        vote.committee.member(issuer).map_or(0, |member| {
            (member.index() + vote.round - 1) % vote.committee.size()
        })
    } else {
        0
    }
}

impl Default for LeaderRotation {
    fn default() -> Self {
        Self::RoundRobin
    }
}
