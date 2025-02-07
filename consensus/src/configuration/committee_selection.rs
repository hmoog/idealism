use committee::Committee;
use crate::{ConfigInterface, Vote};

pub enum CommitteeSelection<Config: ConfigInterface> {
    FixedCommittee(Committee<Config::IssuerID>),
    Custom(fn(&Config, Option<&Vote<Config>>) -> Committee<Config::IssuerID>),
}

impl<Config: ConfigInterface> CommitteeSelection<Config> {
    pub fn dispatch(&self, config: &Config, vote: Option<&Vote<Config>>) -> Committee<Config::IssuerID> {
        match self {
            Self::FixedCommittee(committee) => fixed_committee(committee, vote),
            Self::Custom(strategy) => strategy(config, vote),
        }
    }
}

pub fn fixed_committee<Config: ConfigInterface>(
    committee: &Committee<Config::IssuerID>,
    vote: Option<&Vote<Config>>,
) -> Committee<Config::IssuerID> {
    match vote {
        Some(vote) => vote.committee.clone(),
        None => (*committee).clone(),
    }
}
