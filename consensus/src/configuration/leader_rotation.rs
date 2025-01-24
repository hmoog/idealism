use crate::{VoteData, configuration::config::Config};

pub enum LeaderRotation {
    RoundRobin,
    Custom(fn(&Config, &VoteData<Config>) -> u64),
}

impl LeaderRotation {
    pub fn dispatch(&self, config: &Config, vote: &VoteData<Config>) -> u64 {
        match self {
            Self::RoundRobin => round_robin(vote),
            Self::Custom(strategy) => strategy(config, vote),
        }
    }
}

fn round_robin(vote: &VoteData<Config>) -> u64 {
    vote.committee.member(&vote.issuer).map_or(0, |member| {
        (member.index() + vote.round) % vote.committee.size()
    })
}
