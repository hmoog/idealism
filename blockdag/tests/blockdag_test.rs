use blockdag::BlockDAG;
use common::blocks::{Block, NetworkBlock};
use config::Config;

#[test]
fn test_block_dag() {
    let block_dag: BlockDAG<Config> = BlockDAG::default();

    block_dag
        .subscribe(|metadata| {
            println!("Block {} is ready", metadata.block.id());
        })
        .retain();

    block_dag.queue(Block::from(NetworkBlock {
        parents: vec![],
        issuer_id: Default::default(),
    }));

    block_dag.queue(Block::from(NetworkBlock {
        parents: vec![],
        issuer_id: Default::default(),
    }));
}
