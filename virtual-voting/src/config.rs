use committee::{Committee, MemberID};

use crate::{Vote, VoteBuilder};

pub trait Config: Default {
    type IssuerID: MemberID;

    fn genesis_time(&self) -> u64;

    fn slot_oracle(&self, vote: &VoteBuilder<Self>) -> u64;

    fn offline_threshold(&self) -> u64;

    fn select_committee(&self, vote: Option<&Vote<Self>>) -> Committee<Self::IssuerID>;

    fn leader_weight(&self, vote: &VoteBuilder<Self>) -> u64;
}
