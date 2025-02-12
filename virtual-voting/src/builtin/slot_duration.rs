use crate::{Config, DefaultConfig, VoteBuilder};

pub enum SlotDuration {
    Static(u64),
    Dynamic(fn(&DefaultConfig, &VoteBuilder<DefaultConfig>) -> u64),
}

impl SlotDuration {
    pub fn map_slot(&self, config: &DefaultConfig, vote: &VoteBuilder<DefaultConfig>) -> u64 {
        match self {
            Self::Static(duration) => vote.time - config.genesis_time() / duration,
            Self::Dynamic(strategy) => strategy(config, vote),
        }
    }
}
