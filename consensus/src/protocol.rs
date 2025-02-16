use blockdag::{Block, BlockDAG};
use virtual_voting::{Config, Vote};

pub struct Protocol<C: Config, B: Block> {
    config: C,
    genesis: Vote<C>,
    block_dag: BlockDAG<B>,
}