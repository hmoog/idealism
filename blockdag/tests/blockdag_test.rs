use std::fmt::{Debug, Formatter};

use blockdag::{Block, BlockDAG};

struct MyBlock {
    id: String,
    parents: Vec<String>,
}

impl Debug for MyBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MyBlock")
            .field("id", &self.id)
            .field("parents", &self.parents)
            .finish()
    }
}

impl Block for MyBlock {
    type ID = String;

    fn id(&self) -> &Self::ID {
        &self.id
    }

    fn parents(&self) -> &[Self::ID] {
        &self.parents
    }
}

#[test]
fn test_block_dag() {
    let block_dag: BlockDAG<MyBlock> = BlockDAG::new();

    block_dag
        .on_ready(|metadata| {
            println!("Block {} is ready", metadata.block().id());
        })
        .forever();

    block_dag.queue(MyBlock {
        id: String::from("block2"),
        parents: vec![String::from("block1")],
    });

    block_dag.queue(MyBlock {
        id: String::from("block1"),
        parents: vec![],
    });
}
