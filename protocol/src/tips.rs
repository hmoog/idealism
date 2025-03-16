use blockdag::BlockMetadata;
use types::BlockID;
use utils::rx::Variable;
use virtual_voting::Config;

pub struct Tips<C: Config> {
    _heaviest: Variable<BlockMetadata<C>>,
}

impl<C: Config> Tips<C> {
    pub fn new() -> Self {
        Self {
            _heaviest: Variable::new(),
        }
    }

    pub fn get(&self) -> Vec<BlockID> {
        vec![]
    }
}
