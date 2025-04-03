use blockdag::BlockMetadata;

use crate::{ProtocolConfig, ProtocolResult};

pub trait ProtocolPlugin<C: ProtocolConfig>: Send + Sync {
    fn process_block(&self, block: &BlockMetadata<C>) -> ProtocolResult<()>;
}
