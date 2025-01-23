mod bft_committee;
mod errors;
mod consensus;

pub use bft_committee::*;

mod configuration {
    pub mod leader_rotation;
    pub mod committee_selection;
    pub mod config_interface;
    pub mod config;
}

pub use crate::configuration::{
    committee_selection::CommitteeSelection,
    config::Config,
    config_interface::ConfigInterface,
    leader_rotation::LeaderRotation,
};

pub(crate) mod voting {
    pub(crate) mod vote;
    pub(crate) mod vote_data;
    pub(crate) mod votes;
    pub(crate) mod vote_ref;
    pub(crate) mod votes_by_issuer;
    pub(crate) mod vote_refs;
    pub(crate) mod vote_refs_by_issuer;
    pub(crate) mod votes_by_round;
}

pub use crate::voting::{
    vote::Vote,
    votes::Votes,
    vote_data::VoteData,
    vote_ref::VoteRef,
    votes_by_issuer::VotesByIssuer,
    vote_refs::VoteRefs,
    vote_refs_by_issuer::VoteRefsByIssuer,
    votes_by_round::VotesByRound,
};

#[cfg(test)]
mod test {
    use crate::Config;
    use crate::errors::Error;
    use crate::Vote;

    #[test]
    fn test_consensus() -> Result<(), Error> {
        let genesis = Vote::new_genesis(Config::new());

        let members = genesis.committee().members();

        let member1_vote_1_1 = Vote::aggregate(members[0].key(), vec![&genesis])?;
        let member2_vote_1_1 = Vote::aggregate(members[1].key(), vec![&genesis])?;
        let member3_vote_1_1 = Vote::aggregate(members[2].key(), vec![&genesis])?;
        let member4_vote_1_1 = Vote::aggregate(members[3].key(), vec![&genesis])?;
        assert!(member1_vote_1_1.target().is(&genesis));
        assert!(member2_vote_1_1.target().is(&genesis));
        assert!(member3_vote_1_1.target().is(&genesis));
        assert!(member4_vote_1_1.target().is(&genesis));

        let a2 = Vote::aggregate(members[0].key(), vec![&member1_vote_1_1, &member2_vote_1_1, &member3_vote_1_1])?;
        let b2 = Vote::aggregate(members[1].key(), vec![&member1_vote_1_1, &member2_vote_1_1, &member3_vote_1_1])?;
        let c2 = Vote::aggregate(members[2].key(), vec![&member1_vote_1_1, &member2_vote_1_1, &member3_vote_1_1, &member4_vote_1_1])?;
        assert!(a2.target().is(&member3_vote_1_1));
        assert!(b2.target().is(&member3_vote_1_1));
        assert!(c2.target().is(&member4_vote_1_1));

        Ok(())
    }
}