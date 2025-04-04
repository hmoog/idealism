use crate::{BlockDAGParams, ProtocolParams, VirtualVotingParams};

#[derive(Default)]
pub struct Config {
    pub(crate) protocol_params: ProtocolParams,
    pub(crate) blockdag_params: BlockDAGParams,
    pub(crate) virtual_voting_params: VirtualVotingParams,
}

impl Config {
    pub fn with_protocol_params(mut self, protocol_params: ProtocolParams) -> Self {
        self.protocol_params = protocol_params;
        self
    }
}
