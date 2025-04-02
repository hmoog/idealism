use blockdag::BlockDAG;
use common::blocks::{Block, NetworkBlock};
use config::Config;

#[test]
fn test_block_dag() {
    let block_dag: BlockDAG<Config> = BlockDAG::default();

    block_dag
        .on_block_ready(|metadata| {
            println!("Block {} is ready", metadata.block.id());
        })
        .forever();

    block_dag.attach(Block::from(NetworkBlock {
        parents: vec![],
        issuer_id: Default::default(),
    }));

    block_dag.attach(Block::from(NetworkBlock {
        parents: vec![],
        issuer_id: Default::default(),
    }));
}
