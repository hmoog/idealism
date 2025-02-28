pub use crate::{
    collections::*, config::*, error::*, issuer::*, milestone::*, virtual_voting::*, vote::*,
    vote_builder::*, vote_ref::*, weight_tracker::*,
};
pub mod builtin {
    mod committee_selection;
    mod default_config;
    mod leader_rotation;
    mod slot_duration;

    pub use committee_selection::*;
    pub use default_config::*;
    pub use leader_rotation::*;
    pub use slot_duration::*;
}
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
mod config;
mod error;
mod issuer;
mod milestone;
mod virtual_voting;
mod vote;
mod vote_builder;
mod vote_ref;
mod weight_tracker;

#[cfg(test)]
mod test {
    use types::{BlockID, Hashable, Hasher};

    use crate::{Result, Vote, Votes, builtin::DefaultConfig};

    pub struct Block(u64);

    impl Hashable for Block {
        fn hash<H: Hasher>(&self, hasher: &mut H) {
            hasher.update(&self.0.to_be_bytes());
        }
    }

    pub struct BlockIDGenerator(u64);

    impl BlockIDGenerator {
        pub fn new() -> Self {
            Self(0)
        }

        pub fn next(&mut self) -> BlockID {
            self.0 += 1;
            Block(self.0).into()
        }
    }

    #[test]
    fn test_consensus() -> Result<()> {
        let mut block_id = BlockIDGenerator::new();

        let genesis = Vote::new_genesis(DefaultConfig::new());
        let members = genesis.committee.members();

        println!("FIRST ROUND - VOTE FOR GENESIS");

        let vote1_1 = Vote::new(
            block_id.next(),
            members[0].key(),
            1,
            Votes::from_iter(vec![genesis.clone()]),
        )?;
        let vote2_1 = Vote::new(
            block_id.next(),
            members[1].key(),
            1,
            Votes::from_iter(vec![genesis.clone()]),
        )?;
        let vote3_1 = Vote::new(
            block_id.next(),
            members[2].key(),
            1,
            Votes::from_iter(vec![genesis.clone()]),
        )?;
        let vote4_1 = Vote::new(
            block_id.next(),
            members[3].key(),
            1,
            Votes::from_iter(vec![genesis.clone()]),
        )?;
        assert!(vote1_1.milestone()?.prev.points_to(&genesis));
        assert!(vote2_1.milestone()?.prev.points_to(&genesis));
        assert!(vote3_1.milestone()?.prev.points_to(&genesis));
        assert!(vote4_1.milestone()?.prev.points_to(&genesis));
        println!("{:?}: {:?}", vote1_1, vote1_1.milestone()?.accepted,);
        println!("{:?}: {:?}", vote2_1, vote2_1.milestone()?.accepted,);
        println!("{:?}: {:?}", vote3_1, vote3_1.milestone()?.accepted,);
        println!("{:?}: {:?}", vote4_1, vote4_1.milestone()?.accepted,);

        println!("SECOND ROUND");

        let vote1_2 = Vote::new(
            block_id.next(),
            members[0].key(),
            2,
            Votes::from_iter(vec![vote1_1.clone(), vote2_1.clone(), vote3_1.clone()]),
        )?;
        let vote2_2 = Vote::new(
            block_id.next(),
            members[1].key(),
            2,
            Votes::from_iter(vec![vote1_1.clone(), vote2_1.clone(), vote3_1.clone()]),
        )?;
        let vote3_2 = Vote::new(
            block_id.next(),
            members[2].key(),
            2,
            Votes::from_iter(vec![vote1_1.clone(), vote2_1.clone(), vote3_1.clone()]),
        )?;
        let vote4_2 = Vote::new(
            block_id.next(),
            members[3].key(),
            2,
            Votes::from_iter(vec![
                vote1_1.clone(),
                vote2_1.clone(),
                vote3_1.clone(),
                vote4_1.clone(),
            ]),
        )?;
        assert!(vote1_2.milestone()?.prev.points_to(&vote3_1));
        assert!(vote2_2.milestone()?.prev.points_to(&vote3_1));
        // assert!(vote3_2.last_accepted_milestone_view.commitment()?.heaviest_tip.points_to(&
        // vote4_1));
        println!("{:?}: {:?}", vote1_2, vote1_2.milestone()?.accepted,);
        println!("{:?}: {:?}", vote2_2, vote2_2.milestone()?.accepted,);
        println!("{:?}: {:?}", vote3_2, vote3_2.milestone()?.accepted,);
        println!("{:?}: {:?}", vote4_2, vote4_2.milestone()?.accepted,);

        println!("THIRD ROUND");

        let vote1_3 = Vote::new(
            block_id.next(),
            members[0].key(),
            3,
            Votes::from_iter(vec![
                vote1_2.clone(),
                vote2_2.clone(),
                vote3_2.clone(),
                vote4_2.clone(),
            ]),
        )?;
        let vote2_3 = Vote::new(
            block_id.next(),
            members[1].key(),
            3,
            Votes::from_iter(vec![
                vote1_2.clone(),
                vote2_2.clone(),
                vote3_2.clone(),
                vote4_2.clone(),
            ]),
        )?;
        let vote3_3 = Vote::new(
            block_id.next(),
            members[2].key(),
            3,
            Votes::from_iter(vec![
                vote1_2.clone(),
                vote2_2.clone(),
                vote3_2.clone(),
                vote4_2.clone(),
            ]),
        )?;
        println!("{:?}: {:?}", vote1_3, vote1_3.milestone()?.accepted,);
        println!("{:?}: {:?}", vote2_3, vote2_3.milestone()?.accepted,);
        println!("{:?}: {:?}", vote3_3, vote3_3.milestone()?.accepted,);

        println!("FOURTH ROUND");

        let member1_vote_4 = Vote::new(
            block_id.next(),
            members[0].key(),
            4,
            Votes::from_iter(vec![vote1_3.clone(), vote2_3.clone(), vote3_3.clone()]),
        )?;
        let member2_vote_4 = Vote::new(
            block_id.next(),
            members[0].key(),
            4,
            Votes::from_iter(vec![vote1_3.clone(), vote2_3.clone(), vote3_3.clone()]),
        )?;
        let member3_vote_4 = Vote::new(
            block_id.next(),
            members[0].key(),
            4,
            Votes::from_iter(vec![vote1_3.clone(), vote2_3.clone(), vote3_3.clone()]),
        )?;
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
