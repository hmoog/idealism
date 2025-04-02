use virtual_voting::Vote;

use crate::{Protocol, ProtocolConfig, Result};

pub trait ProtocolPlugin<C: ProtocolConfig>: Send + Sync {
    fn process_vote(&self, protocol: &Protocol<C>, vote: &Vote<C>) -> Result<()>;
}
