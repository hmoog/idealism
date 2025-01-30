pub(crate) use crate::bft_committee::committee_data::CommitteeData;
pub use crate::{
    bft_committee::{
        committee::Committee, committee_member::CommitteeMember,
        committee_member_id::CommitteeMemberID,
    },
    configuration::{
        committee_selection::CommitteeSelection, config::Config, config_interface::ConfigInterface,
        leader_rotation::LeaderRotation,
    },
    errors::{Error, Result},
    voting::{
        issuer::Issuer, vote::Vote, vote_data::VoteData, vote_ref::VoteRef, vote_refs::VoteRefs,
        vote_refs_by_issuer::VoteRefsByIssuer, votes::Votes, votes_by_issuer::VotesByIssuer,
        votes_by_round::VotesByRound,
    },
};

pub(crate) mod consensus;

pub(crate) mod errors;

pub(crate) mod bft_committee {
    pub(crate) mod committee;
    pub(crate) mod committee_data;
    pub(crate) mod committee_member;
    pub(crate) mod committee_member_id;
}

pub(crate) mod configuration {
    pub(crate) mod committee_selection;
    pub(crate) mod config;
    pub(crate) mod config_interface;
    pub(crate) mod leader_rotation;
}

pub(crate) mod voting {
    pub(crate) mod issuer;
    pub(crate) mod vote;
    pub(crate) mod vote_data;
    pub(crate) mod vote_ref;
    pub(crate) mod vote_refs;
    pub(crate) mod vote_refs_by_issuer;
    pub(crate) mod votes;
    pub(crate) mod votes_by_issuer;
    pub(crate) mod votes_by_round;
}

pub(crate) mod utils {
    pub(crate) mod set;
    pub(crate) mod set_element;
}

#[cfg(test)]
mod test {
    use crate::{Config, Vote, errors::Error};

    #[test]
    fn test_consensus() -> Result<(), Error> {
        let genesis = Vote::from(Config::new());

        let members = genesis.committee.members();

        let member1_vote_1_1 = Vote::new(members[0].key(), vec![&genesis])?;
        let member2_vote_1_1 = Vote::new(members[1].key(), vec![&genesis])?;
        let member3_vote_1_1 = Vote::new(members[2].key(), vec![&genesis])?;
        let member4_vote_1_1 = Vote::new(members[3].key(), vec![&genesis])?;
        assert!(member1_vote_1_1.target.points_to(&genesis));
        assert!(member2_vote_1_1.target.points_to(&genesis));
        assert!(member3_vote_1_1.target.points_to(&genesis));
        assert!(member4_vote_1_1.target.points_to(&genesis));

        let a2 = Vote::new(members[0].key(), vec![
            &member1_vote_1_1,
            &member2_vote_1_1,
            &member3_vote_1_1,
        ])?;
        let b2 = Vote::new(members[1].key(), vec![
            &member1_vote_1_1,
            &member2_vote_1_1,
            &member3_vote_1_1,
        ])?;
        let c2 = Vote::new(members[2].key(), vec![
            &member1_vote_1_1,
            &member2_vote_1_1,
            &member3_vote_1_1,
            &member4_vote_1_1,
        ])?;
        assert!(a2.target.points_to(&member3_vote_1_1));
        assert!(b2.target.points_to(&member3_vote_1_1));
        assert!(c2.target.points_to(&member4_vote_1_1));

        Ok(())
    }
}
