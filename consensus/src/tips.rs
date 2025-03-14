use blockdag::BlockMetadata;
use types::BlockID;
use utils::rx::Variable;

use crate::block::Block;

pub struct Tips {
    _heaviest: Variable<BlockMetadata<Block>>,
}

impl Tips {
    pub fn new() -> Self {
        Self {
            _heaviest: Variable::new(),
        }
    }

    pub fn get(&self) -> Vec<BlockID> {
        vec![]
    }
}
