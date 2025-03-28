use blockdag::BlockDAGConfig;
use virtual_voting::VirtualVotingConfig;

use crate::Error;

pub trait ProtocolConfig: VirtualVotingConfig + BlockDAGConfig<ErrorType = Error> {}
