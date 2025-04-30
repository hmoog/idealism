use common::bft::Committee;
use protocol::ProtocolConfig;

use crate::{Vote, VoteBuilder};

pub trait VirtualVotingConfig: ProtocolConfig + Default + Sync + Send + 'static {
    type Source: Send + Sync;

    fn genesis_time(&self) -> u64;

    fn slot_oracle(&self, time: u64) -> u64;

    fn offline_threshold(&self) -> u64;

    fn select_committee(&self, vote: Option<&Vote<Self>>) -> Committee;

    fn leader_weight(&self, vote: &VoteBuilder<Self>) -> u64;
}
