use common::rx::ResourceGuard;

use crate::{BlockDAGConfig, BlockDAGResult, BlockMetadata};

pub trait BlockDAGPlugin<C: BlockDAGConfig>: Send + Sync {
    fn process_block(&self, block: &ResourceGuard<BlockMetadata<C>>) -> BlockDAGResult<()>;
}
