use common::ids::IssuerID;
use config::{Config, ProtocolParams, ProtocolPlugins};
use protocol::{Protocol, ProtocolResult};
use protocol_plugins::{
    block_factory::BlockFactory, consensus_feed::ConsensusFeed, consensus_round::ConsensusRound,
};
use virtual_voting::Vote;

#[test]
fn test_protocol() -> ProtocolResult<()> {
    let protocol = Protocol::new(Config::default().with_protocol_params(
        ProtocolParams::default().with_plugins(ProtocolPlugins::Custom(|cfg, registry| {
            ProtocolPlugins::Core.inject(cfg, registry);

            // define anonymous logging functionality (that subscribes on init)
            registry
                .load::<ConsensusFeed<Config>>()
                .subscribe(|event| println!("consensus: {:?}", event))
                .retain();
        })),
    ));

    let consensus_round = protocol.plugins.get::<ConsensusRound<Config>>().unwrap();
    let block_factory = protocol.plugins.get::<BlockFactory<Config>>().unwrap();

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

    let block_1 = block_factory.new_block(&IssuerID::from([1u8; 32]));
    let block_2 = block_factory.new_block(&IssuerID::from([2u8; 32]));
    let block_3 = block_factory.new_block(&IssuerID::from([3u8; 32]));
    let block_4 = block_factory.new_block(&IssuerID::from([4u8; 32]));

    let block1_metadata = protocol.block_dag.queue(block_1);
    let block2_metadata = protocol.block_dag.queue(block_2);
    let _block3_metadata = protocol.block_dag.queue(block_3);
    let _block4_metadata = protocol.block_dag.queue(block_4);

    println!(
        "{}",
        block1_metadata
            .try_get::<Vote<Config>>()?
            .milestone()?
            .height
    );
    println!(
        "{}",
        block2_metadata
            .try_get::<Vote<Config>>()?
            .milestone()
            .is_ok()
    );

    let block_1_1 = block_factory.new_block(&IssuerID::from([1u8; 32]));
    let block_1_1_metadata = protocol.block_dag.queue(block_1_1);

    println!(
        "{}",
        block_1_1_metadata
            .try_get::<Vote<Config>>()?
            .accepted_milestone()?
            .height()?
    );

    Ok(())
}
