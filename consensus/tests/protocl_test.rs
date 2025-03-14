use consensus::{IssuerID, Protocol};
use virtual_voting::builtin::DefaultConfig;

#[test]
fn test_protocol() {
    let protocol = Protocol::new(DefaultConfig::new());
    protocol.run();

    let _ = protocol
        .blocks_ordered
        .subscribe(|event| println!("Blocks ordered: {:?}", event));

    let issuer_id = IssuerID::from([1; 32]);

    protocol.issue_block(&issuer_id);
}
