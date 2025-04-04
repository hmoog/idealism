use common::ids::BlockID;

use crate::Config;

#[derive(Default)]
pub struct BlockDAGParams {
    genesis_block_id: BlockID,
}

impl blockdag::BlockDAGConfig for Config {
    type ErrorType = protocol::ProtocolError;

    fn genesis_block_id(&self) -> BlockID {
        self.blockdag_params.genesis_block_id.clone()
    }
}
