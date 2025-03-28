use virtual_voting::VirtualVotingConfig as _;

use crate::Config;

pub enum SlotDuration {
    Static(u64),
    Dynamic(fn(&Config, u64) -> u64),
}

impl SlotDuration {
    pub fn map_slot(&self, config: &Config, time: u64) -> u64 {
        match self {
            Self::Static(duration) => time - config.genesis_time() / duration,
            Self::Dynamic(strategy) => strategy(config, time),
        }
    }
}
