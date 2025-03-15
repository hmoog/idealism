use protocol::Protocol;
use types::IssuerID;
use virtual_voting::builtin::DefaultConfig;

#[test]
fn test_protocol() {
    let protocol = Protocol::new(DefaultConfig::new());
    protocol.run();

    let _ = protocol
        .blocks_ordered
        .subscribe(|event| println!("Blocks ordered: {:?}", event));

    protocol.issue_block(&IssuerID::from([1; 32]));
}
