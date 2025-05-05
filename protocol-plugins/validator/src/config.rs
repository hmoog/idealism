use common::ids::IssuerID;
use config::Config;
use protocol::ProtocolConfig;
use virtual_voting::VirtualVotingConfig;

use crate::ValidatorConfigParams;

pub trait ValidatorConfig: VirtualVotingConfig {
    fn validator_id(&self) -> IssuerID;
}

impl ValidatorConfig for Config {
    fn validator_id(&self) -> IssuerID {
        let params = self
            .params::<ValidatorConfigParams>()
            .expect("ValidatorConfigParams not found in config");

        params.validator_id.clone()
    }
}
