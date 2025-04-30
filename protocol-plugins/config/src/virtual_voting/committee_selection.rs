use common::{
    bft::{Committee, Member},
    ids::IssuerID,
};
use virtual_voting::{VirtualVotingConfig, Vote};

pub enum CommitteeSelection<C: VirtualVotingConfig> {
    FixedCommittee(Committee),
    Custom(CustomStrategy<C>),
}

impl<C: VirtualVotingConfig> CommitteeSelection<C> {
    pub fn dispatch(&self, config: &C, vote: Option<&Vote<C>>) -> Committee {
        match self {
            Self::FixedCommittee(committee) => fixed_committee(committee, vote),
            Self::Custom(strategy) => strategy(config, vote),
        }
    }
}

fn fixed_committee<C: VirtualVotingConfig>(
    committee: &Committee,
    vote: Option<&Vote<C>>,
) -> Committee {
    match vote {
        Some(vote) => vote.committee.clone(),
        None => (*committee).clone(),
    }
}

type CustomStrategy<C> = fn(&C, Option<&Vote<C>>) -> Committee;

impl<C: VirtualVotingConfig> Default for CommitteeSelection<C> {
    fn default() -> Self {
        CommitteeSelection::FixedCommittee(Committee::from([
            Member::new(IssuerID::from([1u8; 32])),
            Member::new(IssuerID::from([2u8; 32])),
            Member::new(IssuerID::from([3u8; 32])),
            Member::new(IssuerID::from([4u8; 32])),
        ]))
    }
}
