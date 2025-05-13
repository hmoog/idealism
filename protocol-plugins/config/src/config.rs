use tracing::{info_span, Span};
use protocol::Plugin;
use crate::{ProtocolParams, VirtualVotingParams};

pub struct Config {
    pub protocol_params: ProtocolParams,
    pub virtual_voting_params: VirtualVotingParams,
    span: Span,
}

impl Config {
    pub fn with_protocol_params(mut self, protocol_params: ProtocolParams) -> Self {
        self.protocol_params = protocol_params;
        self
    }
}

impl Plugin for Config {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            protocol_params: ProtocolParams::default(),
            virtual_voting_params: VirtualVotingParams::default(),
            span: info_span!("config"),
        }
    }
}