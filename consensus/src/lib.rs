mod committee;
mod committee_member;
mod committee_member_id;
mod vote;
mod votes;
mod vote_ref;
mod votes_by_issuer;
mod vote_refs;
mod votes_by_round;
mod error;
mod consensus;
mod config;

#[cfg(test)]
mod test {
    use crate::committee_member::CommitteeMember;
    use crate::config::DefaultConfig;
    use crate::error::Error;
    use crate::vote::Vote;

    #[test]
    fn test_consensus() -> Result<(), Error> {
        let genesis = Vote::new_genesis(DefaultConfig).with_alias("genesis");

        let member1 = CommitteeMember::new(1);
        let member2 = CommitteeMember::new(2);
        let member3 = CommitteeMember::new(3);
        let member4 = CommitteeMember::new(4);

        let member1_vote_1_1 = Vote::aggregate(member1.key(), vec![&genesis])?.with_alias("a1");
        let member2_vote_1_1 = Vote::aggregate(member2.key(), vec![&genesis])?.with_alias("b1");
        let member3_vote_1_1 = Vote::aggregate(member3.key(), vec![&genesis])?.with_alias("c1");
        let member4_vote_1_1 = Vote::aggregate(member4.key(), vec![&genesis])?.with_alias("d1");
        assert!(member1_vote_1_1.target().is(&genesis));
        assert!(member2_vote_1_1.target().is(&genesis));
        assert!(member3_vote_1_1.target().is(&genesis));
        assert!(member4_vote_1_1.target().is(&genesis));

        let a2 = Vote::aggregate(member1.key(), vec![&member1_vote_1_1, &member2_vote_1_1, &member3_vote_1_1])?;
        let b2 = Vote::aggregate(member2.key(), vec![&member1_vote_1_1, &member2_vote_1_1, &member3_vote_1_1])?;
        let c2 = Vote::aggregate(member3.key(), vec![&member1_vote_1_1, &member2_vote_1_1, &member3_vote_1_1, &member4_vote_1_1])?;
        assert!(a2.target().is(&member3_vote_1_1));
        assert!(b2.target().is(&member3_vote_1_1));
        assert!(c2.target().is(&member4_vote_1_1));

        Ok(())
    }
}