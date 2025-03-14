use committee::Committee;

use crate::{Vote, VoteBuilder};

pub trait Config: Default + Sync + Send + 'static {
    fn genesis_time(&self) -> u64;

    fn slot_oracle(&self, time: u64) -> u64;

    fn offline_threshold(&self) -> u64;

    fn select_committee(&self, vote: Option<&Vote<Self>>) -> Committee;

    fn leader_weight(&self, vote: &VoteBuilder<Self>) -> u64;
}
