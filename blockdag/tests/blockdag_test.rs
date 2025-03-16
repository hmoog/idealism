use blockdag::{BlockDAG};
use types::{Block, NetworkBlock};

#[test]
fn test_block_dag() {
    let block_dag = BlockDAG::new();

    block_dag
        .on_ready(|metadata| {
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
