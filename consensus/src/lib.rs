mod committee;
mod error;
mod consensus;
mod config;
mod voting;

pub use committee::*;
pub use config::*;

#[cfg(test)]
mod test {
    use crate::config::Config;
    use crate::error::Error;
    use crate::voting::Vote;

    #[test]
    fn test_consensus() -> Result<(), Error> {
        let genesis = Vote::new_genesis(Config::new()).with_alias("genesis");

        let members = genesis.committee().members();

        let member1_vote_1_1 = Vote::aggregate(members[0].key(), vec![&genesis])?.with_alias("a1");
        let member2_vote_1_1 = Vote::aggregate(members[1].key(), vec![&genesis])?.with_alias("b1");
        let member3_vote_1_1 = Vote::aggregate(members[2].key(), vec![&genesis])?.with_alias("c1");
        let member4_vote_1_1 = Vote::aggregate(members[3].key(), vec![&genesis])?.with_alias("d1");
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