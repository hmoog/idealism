use blockdag::BlockDAG;
use types::blocks::{Block, NetworkBlock};
use virtual_voting::builtin::DefaultConfig;

#[test]
fn test_block_dag() {
    let block_dag: BlockDAG<DefaultConfig> = BlockDAG::new();

    block_dag
        .on_block_ready(|metadata| {
            println!("Block {} is ready", metadata.block.id());
        })
        .forever();

    block_dag.queue(Block::from(NetworkBlock {
        parents: vec![],
        issuer_id: Default::default(),
    }));

    block_dag.queue(Block::from(NetworkBlock {
        parents: vec![],
        issuer_id: Default::default(),
    }));
}
