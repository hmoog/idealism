pub use crate::{
    configuration::{
        committee_selection::CommitteeSelection, config::Config, config_interface::ConfigInterface,
        leader_rotation::LeaderRotation,
    },
    consensus::*,
    errors::*,
    voting::{
        issuer::Issuer, vote::Vote, vote_builder::VoteBuilder, vote_ref::VoteRef,
        vote_refs::VoteRefs, vote_refs_by_issuer::VoteRefsByIssuer, votes::Votes,
        votes_by_issuer::VotesByIssuer, votes_by_round::VotesByRound,
    },
};

mod consensus {
    mod consensus_commitment;
    mod consensus_mechanism;
    mod vote_tracker;

    pub use consensus_commitment::ConsensusCommitment;
    pub use consensus_mechanism::ConsensusMechanism;
}

pub(crate) mod errors;

pub(crate) mod configuration {
    pub(crate) mod committee_selection;
    pub(crate) mod config;
    pub(crate) mod config_interface;
    pub(crate) mod leader_rotation;
    mod slot_duration;

    pub use slot_duration::SlotDuration;
}

pub(crate) mod voting {
    pub(crate) mod issuer;
    pub(crate) mod vote;
    pub(crate) mod vote_builder;
    pub(crate) mod vote_ref;
    pub(crate) mod vote_refs;
    pub(crate) mod vote_refs_by_issuer;
    pub(crate) mod votes;
    pub(crate) mod votes_by_issuer;
    pub(crate) mod votes_by_round;
}

#[cfg(test)]
mod test {
    use crate::{Config, Result, Vote};

    #[test]
    fn test_consensus() -> Result<()> {
        let genesis = Vote::from(Config::new());
        let members = genesis.committee.members();

        println!("FIRST ROUND - VOTE FOR GENESIS");

        let vote1_1 = Vote::new(members[0].key(), 1, vec![&genesis])?;
        let vote2_1 = Vote::new(members[1].key(), 1, vec![&genesis])?;
        let vote3_1 = Vote::new(members[2].key(), 1, vec![&genesis])?;
        let vote4_1 = Vote::new(members[3].key(), 1, vec![&genesis])?;
        assert!(
            vote1_1
                .consensus
                .heaviest_tip
                .points_to(&genesis)
        );
        assert!(
            vote2_1
                .consensus
                .heaviest_tip
                .points_to(&genesis)
        );
        assert!(
            vote3_1
                .consensus
                .heaviest_tip
                .points_to(&genesis)
        );
        assert!(
            vote4_1
                .consensus
                .heaviest_tip
                .points_to(&genesis)
        );
        println!("{:?}: {:?}", vote1_1, vote1_1.consensus,);
        println!("{:?}: {:?}", vote2_1, vote2_1.consensus,);
        println!("{:?}: {:?}", vote3_1, vote3_1.consensus,);
        println!("{:?}: {:?}", vote4_1, vote4_1.consensus,);

        println!("SECOND ROUND");

        let vote1_2 = Vote::new(members[0].key(), 2, vec![&vote1_1, &vote2_1, &vote3_1])?;
        let vote2_2 = Vote::new(members[1].key(), 2, vec![&vote1_1, &vote2_1, &vote3_1])?;
        let vote3_2 = Vote::new(members[2].key(), 2, vec![&vote1_1, &vote2_1, &vote3_1])?;
        let vote4_2 = Vote::new(members[3].key(), 2, vec![
            &vote1_1, &vote2_1, &vote3_1, &vote4_1,
        ])?;
        assert!(
            vote1_2
                .consensus
                .heaviest_tip
                .points_to(&vote3_1)
        );
        assert!(
            vote2_2
                .consensus
                .heaviest_tip
                .points_to(&vote3_1)
        );
        // assert!(vote3_2.consensus_view.heaviest_tip.points_to(&vote4_1));
        println!("{:?}: {:?}", vote1_2, vote1_2.consensus,);
        println!("{:?}: {:?}", vote2_2, vote2_2.consensus,);
        println!("{:?}: {:?}", vote3_2, vote3_2.consensus,);
        println!("{:?}: {:?}", vote4_2, vote4_2.consensus,);

        println!("THIRD ROUND");

        let vote1_3 = Vote::new(members[0].key(), 3, vec![
            &vote1_2, &vote2_2, &vote3_2, &vote4_2,
        ])?;
        let vote2_3 = Vote::new(members[1].key(), 3, vec![
            &vote1_2, &vote2_2, &vote3_2, &vote4_2,
        ])?;
        let vote3_3 = Vote::new(members[2].key(), 3, vec![
            &vote1_2, &vote2_2, &vote3_2, &vote4_2,
        ])?;
        println!("{:?}: {:?}", vote1_3, vote1_3.consensus,);
        println!("{:?}: {:?}", vote2_3, vote2_3.consensus,);
        println!("{:?}: {:?}", vote3_3, vote3_3.consensus,);

        println!("FOURTH ROUND");

        let member1_vote_4 = Vote::new(members[0].key(), 4, vec![&vote1_3, &vote2_3, &vote3_3])?;
        let member2_vote_4 = Vote::new(members[0].key(), 4, vec![&vote1_3, &vote2_3, &vote3_3])?;
        let member3_vote_4 = Vote::new(members[0].key(), 4, vec![&vote1_3, &vote2_3, &vote3_3])?;
        println!(
            "member1_vote_4 (round {:?}): {:?}",
            member1_vote_4.round, member1_vote_4.consensus,
        );
        println!(
            "{:?} {:?}",
            member2_vote_4.consensus.heaviest_tip,
            member2_vote_4.consensus.accepted_milestone
        );
        println!(
            "{:?} {:?}",
            member3_vote_4.consensus.heaviest_tip,
            member3_vote_4.consensus.accepted_milestone
        );

        Ok(())
    }
}
