use consensus::{Committee, CommitteeMember, Config};
use utils::ArcKey;

#[test]
fn test_committee() {
    let committee: Committee<Config> = Committee::from(vec![
        CommitteeMember::new(1).with_weight(10).with_online(true),
        CommitteeMember::new(2).with_weight(20).with_online(false)
    ]);

    // assert initial state
    assert_eq!(committee.total_weight(), 30);
    assert_eq!(committee.online_weight(), 10);
    assert!(committee.is_member_online(&ArcKey::new(1)));
    assert_eq!(committee.member_weight(&ArcKey::new(1)), 10);
    assert!(!committee.is_member_online(&ArcKey::new(2)));
    assert_eq!(committee.member_weight(&ArcKey::new(2)), 20);

    // set member 2 online
    let committee1 = committee.set_online(&ArcKey::new(2), true);
    assert_eq!(committee1.total_weight(), 30);
    assert_eq!(committee1.online_weight(), 30);
    assert!(committee1.is_member_online(&ArcKey::new(1)));
    assert_eq!(committee1.member_weight(&ArcKey::new(1)), 10);
    assert!(committee1.is_member_online(&ArcKey::new(2)));
    assert_eq!(committee1.member_weight(&ArcKey::new(2)), 20);

    // original committee is not changed
    assert_eq!(committee.total_weight(), 30);
    assert_eq!(committee.online_weight(), 10);
    assert!(committee.is_member_online(&ArcKey::new(1)));
    assert_eq!(committee.member_weight(&ArcKey::new(1)), 10);
    assert!(!committee.is_member_online(&ArcKey::new(2)));
    assert_eq!(committee.member_weight(&ArcKey::new(2)), 20);

    // set member 2 online again (no change / same underlying data)
    //let committee2 = committee1.set_online(&ArcKey::new(2), true);
    //assert!(Arc::ptr_eq(&committee1.0, &committee2.0));
}