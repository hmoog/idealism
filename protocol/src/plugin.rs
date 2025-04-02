use blockdag::BlockMetadata;

use crate::{ProtocolConfig, Result};

pub trait ProtocolPlugin<C: ProtocolConfig>: Send + Sync {
    fn process_block(&self, block: &BlockMetadata<C>) -> Result<()>;
}
