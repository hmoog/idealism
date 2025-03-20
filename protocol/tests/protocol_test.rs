use protocol::Protocol;
use types::ids::IssuerID;
use virtual_voting::builtin::DefaultConfig;

#[test]
fn test_protocol() {
    let protocol = Protocol::new(DefaultConfig::new());

    protocol
        .blocks_ordered
        .subscribe(|event| println!("Blocks ordered: {:?}", event))
        .forever();

    protocol
        .error
        .subscribe(|event| println!("Error: {}", event))
        .forever();

    protocol.issue_block(&IssuerID::from([1; 32]));
    protocol.issue_block(&IssuerID::from([2; 32]));
}
