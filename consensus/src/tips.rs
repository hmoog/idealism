use blockdag::BlockMetadata;
use utils::rx::Variable;
use virtual_voting::Config;
use crate::types::Block;

pub struct Tips<C: Config> {
    heaviest: Variable<BlockMetadata<Block<C>>>
}