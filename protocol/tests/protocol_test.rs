use common::ids::IssuerID;
use config::Config;
use protocol::{Protocol, ProtocolResult};
use protocol_plugins::{
    block_factory::BlockFactory, consensus::Consensus, consensus_round::ConsensusRound,
};

#[test]
fn test_protocol() -> ProtocolResult<()> {
    let protocol = Protocol::new(Config::default());

    let consensus = protocol.plugins.get::<Consensus<Config>>().unwrap();
    let consensus_round = protocol.plugins.get::<ConsensusRound<Config>>().unwrap();
    let block_factory = protocol.plugins.get::<BlockFactory<Config>>().unwrap();

    consensus
        .heaviest_milestone_vote
        .subscribe(|update| {
            println!("heaviest_milestone: {:?} => {:?}", update.0, update.1);
        })
        .retain();

    consensus_round
        .started
        .subscribe(|update| {
            println!("round::started: {:?} => {:?}", update.0, update.1);
        })
        .retain();

    consensus_round
        .completed
        .subscribe(|update| {
            println!("round::completed: {:?} => {:?}", update.0, update.1);
        })
        .retain();

    consensus
        .committee
        .subscribe(|update| {
            println!(
                "committee: {:?} => {:?}",
                update.0.as_ref().map(|x| x.commitment()),
                update.1.as_ref().map(|x| x.commitment())
            );
        })
        .retain();

    consensus
        .accepted_blocks
        .subscribe(|event| println!("Blocks ordered: {:?}", event))
        .retain();

    let block_1 = block_factory.new_block(&IssuerID::from([1u8; 32]));
    let block_2 = block_factory.new_block(&IssuerID::from([2u8; 32]));
    let block_3 = block_factory.new_block(&IssuerID::from([3u8; 32]));
    let block_4 = block_factory.new_block(&IssuerID::from([4u8; 32]));

    let block1_metadata = protocol.block_dag.attach(block_1);
    let block2_metadata = protocol.block_dag.attach(block_2);
    let _block3_metadata = protocol.block_dag.attach(block_3);
    let _block4_metadata = protocol.block_dag.attach(block_4);

    println!("{}", block1_metadata.vote()?.milestone()?.height);
    println!("{}", block2_metadata.vote()?.milestone().is_ok());

    let block_1_1 = block_factory.new_block(&IssuerID::from([1u8; 32]));
    let block_1_1_metadata = protocol.block_dag.attach(block_1_1);

    println!(
        "{}",
        block_1_1_metadata.vote()?.accepted_milestone()?.height()?
    );

    Ok(())
}
