use blockdag::Block;
use blockdag::BlockDAG;

struct MyBlock {
    id: String,
    parents: Vec<String>,
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

    block_dag.on_ready(|block| {
        println!("Block {} is ready", block.id());
    }).forever();

    block_dag.queue(MyBlock {
        id: String::from("block2"),
        parents: vec![String::from("block1")]
    });

    block_dag.queue(MyBlock {
        id: String::from("block1"),
        parents: vec![]
    });
}