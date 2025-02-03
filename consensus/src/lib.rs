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
    consensus::*,
    errors::*,
    voting::{
        issuer::Issuer, vote::Vote, vote_builder::VoteBuilder, vote_ref::VoteRef,
        vote_refs::VoteRefs, vote_refs_by_issuer::VoteRefsByIssuer, votes::Votes,
        votes_by_issuer::VotesByIssuer, votes_by_round::VotesByRound,
    },
};

mod consensus {
    mod consensus_mechanism;
    mod consensus_view;
    mod consensus_view_ref;

    pub use consensus_view::ConsensusView;
    pub use consensus_view_ref::ConsensusViewRef;
}

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

        let member1_vote_1 = Vote::new(members[0].key(), vec![&genesis])?;
        let member2_vote_1 = Vote::new(members[1].key(), vec![&genesis])?;
        let member3_vote_1 = Vote::new(members[2].key(), vec![&genesis])?;
        let member4_vote_1 = Vote::new(members[3].key(), vec![&genesis])?;
        assert!(
            member1_vote_1
                .consensus_view
                .heaviest_tip
                .points_to(&genesis)
        );
        assert!(
            member2_vote_1
                .consensus_view
                .heaviest_tip
                .points_to(&genesis)
        );
        assert!(
            member3_vote_1
                .consensus_view
                .heaviest_tip
                .points_to(&genesis)
        );
        assert!(
            member4_vote_1
                .consensus_view
                .heaviest_tip
                .points_to(&genesis)
        );

        println!("SECOND ROUND");

        let member1_vote_2 = Vote::new(members[0].key(), vec![
            &member1_vote_1,
            &member2_vote_1,
            &member3_vote_1,
        ])?;
        let member2_vote_2 = Vote::new(members[1].key(), vec![
            &member1_vote_1,
            &member2_vote_1,
            &member3_vote_1,
        ])?;
        let member3_vote_2 = Vote::new(members[2].key(), vec![
            &member1_vote_1,
            &member2_vote_1,
            &member3_vote_1,
            &member4_vote_1,
        ])?;
        let member4_vote_2 = Vote::new(members[3].key(), vec![
            &member1_vote_1,
            &member2_vote_1,
            &member3_vote_1,
            &member4_vote_1,
        ])?;

        println!("{:?}", member1_vote_2.consensus_view.heaviest_tip);

        assert!(
            member1_vote_2
                .consensus_view
                .heaviest_tip
                .points_to(&member3_vote_1)
        );
        assert!(
            member2_vote_2
                .consensus_view
                .heaviest_tip
                .points_to(&member3_vote_1)
        );
        assert!(
            member3_vote_2
                .consensus_view
                .heaviest_tip
                .points_to(&member4_vote_1)
        );

        println!("THIRD ROUND");

        let member1_vote_3 = Vote::new(members[0].key(), vec![
            &member1_vote_2,
            &member2_vote_2,
            &member3_vote_2,
            &member4_vote_2,
        ])?;
        let member2_vote_3 = Vote::new(members[0].key(), vec![
            &member1_vote_2,
            &member2_vote_2,
            &member3_vote_2,
            &member4_vote_2,
        ])?;
        let member3_vote_3 = Vote::new(members[0].key(), vec![
            &member1_vote_2,
            &member2_vote_2,
            &member3_vote_2,
            &member4_vote_2,
        ])?;
        println!(
            "{:?}",
            member1_vote_3.consensus_view.latest_accepted_milestone
        );
        println!(
            "{:?}",
            member2_vote_3.consensus_view.latest_accepted_milestone
        );
        println!(
            "{:?}",
            member3_vote_3.consensus_view.latest_accepted_milestone
        );

        println!("FOURTH ROUND");

        let member1_vote_4 = Vote::new(members[0].key(), vec![
            &member1_vote_3,
            &member2_vote_3,
            &member3_vote_3,
        ])?;
        let member2_vote_4 = Vote::new(members[0].key(), vec![
            &member1_vote_3,
            &member2_vote_3,
            &member3_vote_3,
        ])?;
        let member3_vote_4 = Vote::new(members[0].key(), vec![
            &member1_vote_3,
            &member2_vote_3,
            &member3_vote_3,
        ])?;
        println!(
            "{:?}",
            member1_vote_4.consensus_view.latest_accepted_milestone
        );
        println!(
            "{:?}",
            member2_vote_4.consensus_view.latest_accepted_milestone
        );
        println!(
            "{:?}",
            member3_vote_4.consensus_view.latest_accepted_milestone
        );

        Ok(())
    }
}
