use types::{
    bft::{Committee, Member},
    ids::IssuerID,
};

#[test]
fn test_committee() {
    let member_id_1 = IssuerID::from([1; 32]);
    let member_id_2 = IssuerID::from([2; 32]);

    let committee: Committee = Committee::from(vec![
        Member::new(member_id_1.clone())
            .with_weight(10)
            .with_online(true),
        Member::new(member_id_2.clone())
            .with_weight(20)
            .with_online(false),
    ]);

    // assert initial state
    assert_eq!(committee.total_weight(), 30);
    assert_eq!(committee.online_weight(), 10);
    assert!(committee.is_member_online(&member_id_1));
    assert_eq!(committee.member_weight(&member_id_1), 10);
    assert!(!committee.is_member_online(&member_id_2));
    assert_eq!(committee.member_weight(&member_id_2), 20);

    // set member 2 online
    let committee1 = committee.set_online(&member_id_2, true);
    assert_eq!(committee1.total_weight(), 30);
    assert_eq!(committee1.online_weight(), 30);
    assert!(committee1.is_member_online(&member_id_1));
    assert_eq!(committee1.member_weight(&member_id_1), 10);
    assert!(committee1.is_member_online(&member_id_2));
    assert_eq!(committee1.member_weight(&member_id_2), 20);

    // original committee is not changed
    assert_eq!(committee.total_weight(), 30);
    assert_eq!(committee.online_weight(), 10);
    assert!(committee.is_member_online(&member_id_1));
    assert_eq!(committee.member_weight(&member_id_1), 10);
    assert!(!committee.is_member_online(&member_id_2));
    assert_eq!(committee.member_weight(&member_id_2), 20);

    // set member 2 online again (no change / same underlying data)
    // let committee2 = committee1.set_online(&ArcKey::new(2), true);
    // assert!(Arc::ptr_eq(&committee1.0, &committee2.0));
}
