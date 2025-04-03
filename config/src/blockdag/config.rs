use common::ids::BlockID;

use crate::Config;

impl blockdag::BlockDAGConfig for Config {
    type ErrorType = protocol::ProtocolError;

    fn genesis_block_id(&self) -> BlockID {
        self.blockdag_params.genesis_block_id.clone()
    }
}
