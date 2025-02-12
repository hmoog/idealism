use committee::Committee;

use crate::{Config, Vote};

pub enum CommitteeSelection<C: Config> {
    FixedCommittee(Committee<C::IssuerID>),
    Custom(CustomStrategy<C>),
}

impl<C: Config> CommitteeSelection<C> {
    pub fn dispatch(&self, config: &C, vote: Option<&Vote<C>>) -> Committee<C::IssuerID> {
        match self {
            Self::FixedCommittee(committee) => fixed_committee(committee, vote),
            Self::Custom(strategy) => strategy(config, vote),
        }
    }
}

fn fixed_committee<C: Config>(
    committee: &Committee<C::IssuerID>,
    vote: Option<&Vote<C>>,
) -> Committee<C::IssuerID> {
    match vote {
        Some(vote) => vote.committee.clone(),
        None => (*committee).clone(),
    }
}

type CustomStrategy<C> = fn(&C, Option<&Vote<C>>) -> Committee<<C as Config>::IssuerID>;
