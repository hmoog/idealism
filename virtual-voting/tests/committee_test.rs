use committee::{Committee, Member};
use utils::Id;

#[test]
fn test_committee() {
    let committee: Committee<i32> = Committee::from(vec![
        Member::new(1).with_weight(10).with_online(true),
        Member::new(2).with_weight(20).with_online(false),
    ]);

    // assert initial state
    assert_eq!(committee.total_weight(), 30);
    assert_eq!(committee.online_weight(), 10);
    assert!(committee.is_member_online(&Id::new(1)));
    assert_eq!(committee.member_weight(&Id::new(1)), 10);
    assert!(!committee.is_member_online(&Id::new(2)));
    assert_eq!(committee.member_weight(&Id::new(2)), 20);

    // set member 2 online
    let committee1 = committee.set_online(&Id::new(2), true);
    assert_eq!(committee1.total_weight(), 30);
    assert_eq!(committee1.online_weight(), 30);
    assert!(committee1.is_member_online(&Id::new(1)));
    assert_eq!(committee1.member_weight(&Id::new(1)), 10);
    assert!(committee1.is_member_online(&Id::new(2)));
    assert_eq!(committee1.member_weight(&Id::new(2)), 20);

    // original committee is not changed
    assert_eq!(committee.total_weight(), 30);
    assert_eq!(committee.online_weight(), 10);
    assert!(committee.is_member_online(&Id::new(1)));
    assert_eq!(committee.member_weight(&Id::new(1)), 10);
    assert!(!committee.is_member_online(&Id::new(2)));
    assert_eq!(committee.member_weight(&Id::new(2)), 20);

    // set member 2 online again (no change / same underlying data)
    // let committee2 = committee1.set_online(&ArcKey::new(2), true);
    // assert!(Arc::ptr_eq(&committee1.0, &committee2.0));
}
