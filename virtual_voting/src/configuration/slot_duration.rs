use crate::{Config, ConfigInterface, VoteBuilder};

pub enum SlotDuration {
    Static(u64),
    Dynamic(fn(&Config, &VoteBuilder<Config>) -> u64),
}

impl SlotDuration {
    pub fn map_slot(&self, config: &Config, vote: &VoteBuilder<Config>) -> u64 {
        match self {
            Self::Static(duration) => vote.issuing_time - config.genesis_time() / duration,
            Self::Dynamic(strategy) => strategy(config, vote),
        }
    }
}
