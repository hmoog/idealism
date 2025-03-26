use config::Config;
use protocol::{Protocol, Result};
use types::ids::IssuerID;

#[test]
fn test_protocol() -> Result<()> {
    let protocol = Protocol::new(Config::new());

    protocol
        .state
        .heaviest_milestone
        .subscribe(|update| {
            println!("heaviest_milestone: {:?} => {:?}", update.0, update.1);
        })
        .forever();

    protocol
        .state
        .round
        .subscribe(|update| {
            println!("round: {:?} => {:?}", update.0, update.1);
        })
        .forever();

    protocol
        .state
        .committee
        .subscribe(|update| {
            println!(
                "committee: {:?} => {:?}",
                update.0.as_ref().map(|x| x.commitment()),
                update.1.as_ref().map(|x| x.commitment())
            );
        })
        .forever();

    protocol
        .events
        .blocks_ordered
        .subscribe(|event| println!("Blocks ordered: {:?}", event))
        .forever();

    protocol
        .events
        .error
        .subscribe(|event| println!("Error: {}", event))
        .forever();

    let block_1 = protocol.new_block(&IssuerID::from([1u8; 32]));
    let block_2 = protocol.new_block(&IssuerID::from([2u8; 32]));
    let block_3 = protocol.new_block(&IssuerID::from([3u8; 32]));
    let block_4 = protocol.new_block(&IssuerID::from([4u8; 32]));

    let block1_metadata = protocol.block_dag.attach(block_1);
    let block2_metadata = protocol.block_dag.attach(block_2);
    let _block3_metadata = protocol.block_dag.attach(block_3);
    let _block4_metadata = protocol.block_dag.attach(block_4);

    println!("{}", block1_metadata.vote()?.milestone()?.height);
    println!(
        "{}",
        block2_metadata
            .vote
            .get()
            .as_ref()
            .unwrap()
            .milestone()
            .is_ok()
    );

    let block_1_1 = protocol.new_block(&IssuerID::from([1u8; 32]));
    let block_1_1_metadata = protocol.block_dag.attach(block_1_1);

    println!(
        "{}",
        block_1_1_metadata.vote()?.accepted_milestone()?.block_id
    );

    Ok(())
}
