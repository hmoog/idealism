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
    use crate::committee::Committee;
    use crate::committee_member::CommitteeMember;
    use crate::config::{Config, DefaultConfig};
    use crate::error::Error;
    use crate::vote::Vote;

    #[test]
    fn test_consensus() -> Result<(), Error> {
        let config = DefaultConfig;

        struct TestConfig;
        impl Config for TestConfig {
            type CommitteeMemberID = u64;

            fn select_committee(&self, _vote: &Vote<Self::CommitteeMemberID>) -> Committee<Self::CommitteeMemberID> {
                unimplemented!()
            }
        }

        let member1 = CommitteeMember::new(1);
        let member2 = CommitteeMember::new(2);
        let member3 = CommitteeMember::new(3);
        let member4 = CommitteeMember::new(4);

        let committee = Committee::from([
            member1.clone(),
            member2.clone(),
            member3.clone(),
            member4.clone(),
        ]);

        let genesis = Vote::new_genesis(committee).with_alias("genesis");
        let a1 = Vote::aggregate(member1.key(), vec![&genesis])?.with_alias("a1");
        let b1 = Vote::aggregate(member2.key(), vec![&genesis])?.with_alias("b1");
        let c1 = Vote::aggregate(member3.key(), vec![&genesis])?.with_alias("c1");
        let d1 = Vote::aggregate(member4.key(), vec![&genesis])?.with_alias("d1");
        assert!(a1.target().is(&genesis));
        assert!(b1.target().is(&genesis));
        assert!(c1.target().is(&genesis));
        assert!(d1.target().is(&genesis));

        let a2 = Vote::aggregate(member1.key(), vec![&a1, &b1, &c1])?;
        let b2 = Vote::aggregate(member2.key(), vec![&a1, &b1, &c1])?;
        let c2 = Vote::aggregate(member3.key(), vec![&a1, &b1, &c1])?;
        println!("{} {} {}", a2.target().as_vote()?.alias(), a1.alias(), a2.target().is(&a1));
        assert!(a2.target().is(&a1));
        assert!(b2.target().is(&a1));

        Ok(())
    }
}