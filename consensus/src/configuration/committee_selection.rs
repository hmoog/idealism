use crate::{Committee, Config, Vote};

pub enum CommitteeSelection {
    FixedCommittee(Committee<Config>),
    Custom(fn(&Config, Option<&Vote<Config>>) -> Committee<Config>),
}

impl CommitteeSelection {
    pub fn dispatch(&self, config: &Config, vote: Option<&Vote<Config>>) -> Committee<Config> {
        match self {
            Self::FixedCommittee(committee) => fixed_committee(committee, vote),
            Self::Custom(strategy) => strategy(config, vote),
        }
    }
}

pub fn fixed_committee(
    committee: &Committee<Config>,
    vote: Option<&Vote<Config>>,
) -> Committee<Config> {
    match vote {
        Some(vote) => vote.committee.clone(),
        None => (*committee).clone(),
    }
}
