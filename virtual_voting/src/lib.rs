pub use crate::{
    collections::*,
    configuration::{
        committee_selection::CommitteeSelection, config::Config, config_interface::ConfigInterface,
        leader_rotation::LeaderRotation,
    },
    milestone::*,
    types::*,
    virtual_voting::*,
    vote::*,
    vote_builder::*,
    vote_ref::*,
    weight_tracker::*,
};

mod milestone;
mod types;
mod virtual_voting;
mod vote;
mod vote_builder;
mod vote_ref;
mod weight_tracker;

mod collections {
    mod vote_refs;
    mod vote_refs_by_issuer;
    mod votes;
    mod votes_by_issuer;
    mod votes_by_round;

    pub use vote_refs::*;
    pub use vote_refs_by_issuer::*;
    pub use votes::*;
    pub use votes_by_issuer::*;
    pub use votes_by_round::*;
}

pub(crate) mod configuration {
    pub(crate) mod committee_selection;
    pub(crate) mod config;
    pub(crate) mod config_interface;
    pub(crate) mod leader_rotation;
    mod slot_duration;

    pub use slot_duration::SlotDuration;
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
        assert!(vote1_1.milestone()?.prev.points_to(&genesis));
        assert!(vote2_1.milestone()?.prev.points_to(&genesis));
        assert!(vote3_1.milestone()?.prev.points_to(&genesis));
        assert!(vote4_1.milestone()?.prev.points_to(&genesis));
        println!("{:?}: {:?}", vote1_1, vote1_1.milestone()?.accepted,);
        println!("{:?}: {:?}", vote2_1, vote2_1.milestone()?.accepted,);
        println!("{:?}: {:?}", vote3_1, vote3_1.milestone()?.accepted,);
        println!("{:?}: {:?}", vote4_1, vote4_1.milestone()?.accepted,);

        println!("SECOND ROUND");

        let vote1_2 = Vote::new(members[0].key(), 2, vec![&vote1_1, &vote2_1, &vote3_1])?;
        let vote2_2 = Vote::new(members[1].key(), 2, vec![&vote1_1, &vote2_1, &vote3_1])?;
        let vote3_2 = Vote::new(members[2].key(), 2, vec![&vote1_1, &vote2_1, &vote3_1])?;
        let vote4_2 = Vote::new(members[3].key(), 2, vec![
            &vote1_1, &vote2_1, &vote3_1, &vote4_1,
        ])?;
        assert!(vote1_2.milestone()?.prev.points_to(&vote3_1));
        assert!(vote2_2.milestone()?.prev.points_to(&vote3_1));
        // assert!(vote3_2.last_accepted_milestone_view.commitment()?.heaviest_tip.points_to(&
        // vote4_1));
        println!("{:?}: {:?}", vote1_2, vote1_2.milestone()?.accepted,);
        println!("{:?}: {:?}", vote2_2, vote2_2.milestone()?.accepted,);
        println!("{:?}: {:?}", vote3_2, vote3_2.milestone()?.accepted,);
        println!("{:?}: {:?}", vote4_2, vote4_2.milestone()?.accepted,);

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
        println!("{:?}: {:?}", vote1_3, vote1_3.milestone()?.accepted,);
        println!("{:?}: {:?}", vote2_3, vote2_3.milestone()?.accepted,);
        println!("{:?}: {:?}", vote3_3, vote3_3.milestone()?.accepted,);

        println!("FOURTH ROUND");

        let member1_vote_4 = Vote::new(members[0].key(), 4, vec![&vote1_3, &vote2_3, &vote3_3])?;
        let member2_vote_4 = Vote::new(members[0].key(), 4, vec![&vote1_3, &vote2_3, &vote3_3])?;
        let member3_vote_4 = Vote::new(members[0].key(), 4, vec![&vote1_3, &vote2_3, &vote3_3])?;
        println!(
            "member1_vote_4 (round {:?}): {:?}",
            member1_vote_4.round,
            member1_vote_4.milestone()?.accepted,
        );
        println!(
            "{:?} {:?}",
            member2_vote_4.milestone()?.prev,
            member2_vote_4.milestone()?.accepted,
        );
        println!(
            "{:?} {:?}",
            member3_vote_4.milestone()?.prev,
            member3_vote_4.milestone()?.accepted,
        );

        Ok(())
    }
}
