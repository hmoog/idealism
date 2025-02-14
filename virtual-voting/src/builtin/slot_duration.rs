use crate::{Config, builtin::DefaultConfig};

pub enum SlotDuration {
    Static(u64),
    Dynamic(fn(&DefaultConfig, u64) -> u64),
}

impl SlotDuration {
    pub fn map_slot(&self, config: &DefaultConfig, time: u64) -> u64 {
        match self {
            Self::Static(duration) => time - config.genesis_time() / duration,
            Self::Dynamic(strategy) => strategy(config, time),
        }
    }
}
