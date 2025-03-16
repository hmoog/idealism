use blockdag::BlockMetadata;
use types::BlockID;
use utils::rx::Variable;

pub struct Tips {
    _heaviest: Variable<BlockMetadata>,
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
