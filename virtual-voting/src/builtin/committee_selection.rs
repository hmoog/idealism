use committee::Committee;

use crate::{Config, Vote};

pub enum CommitteeSelection<C: Config> {
    FixedCommittee(Committee),
    Custom(CustomStrategy<C>),
}

impl<C: Config> CommitteeSelection<C> {
    pub fn dispatch(&self, config: &C, vote: Option<&Vote<C>>) -> Committee {
        match self {
            Self::FixedCommittee(committee) => fixed_committee(committee, vote),
            Self::Custom(strategy) => strategy(config, vote),
        }
    }
}

fn fixed_committee<C: Config>(committee: &Committee, vote: Option<&Vote<C>>) -> Committee {
    match vote {
        Some(vote) => vote.committee.clone(),
        None => (*committee).clone(),
    }
}

type CustomStrategy<C> = fn(&C, Option<&Vote<C>>) -> Committee;
