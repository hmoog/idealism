use std::any::TypeId;

use block_storage::BlockStorage;
use common::{
    errors::{Error::MetadataNotFound, Result},
    ids::IssuerID,
};
use config::{Config, ProtocolParams, ProtocolPlugins};
use protocol::Protocol;
use protocol_plugins::{
    block_factory::BlockFactory, consensus_feed::ConsensusFeed, consensus_round::ConsensusRound,
};
use virtual_voting::{Milestone, Vote};

#[test]
fn test_protocol() -> Result<()> {
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
    let block_factory = protocol.plugins.get::<BlockFactory>().unwrap();

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

    let block1_metadata = protocol
        .plugins
        .get::<BlockStorage>()
        .unwrap()
        .insert(block_1);
    let block2_metadata = protocol
        .plugins
        .get::<BlockStorage>()
        .unwrap()
        .insert(block_2);
    let _block3_metadata = protocol
        .plugins
        .get::<BlockStorage>()
        .unwrap()
        .insert(block_3);
    let _block4_metadata = protocol
        .plugins
        .get::<BlockStorage>()
        .unwrap()
        .insert(block_4);

    println!(
        "{}",
        block1_metadata
            .try_get::<Vote<Config>>()?
            .milestone()
            .map_err(|_| MetadataNotFound(TypeId::of::<Milestone<Config>>()))?
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
    let block_1_1_metadata = protocol
        .plugins
        .get::<BlockStorage>()
        .unwrap()
        .insert(block_1_1);

    println!(
        "{}",
        block_1_1_metadata
            .try_get::<Vote<Config>>()?
            .accepted_milestone()
            .map_err(|_| MetadataNotFound(TypeId::of::<Milestone<Config>>()))?
            .height()
            .map_err(|_| MetadataNotFound(TypeId::of::<Milestone<Config>>()))?
    );

    Ok(())
}
