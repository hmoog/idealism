use crate::{Config, VoteRef};

pub struct Milestone<C: Config> {
    pub height: u64,
    pub leader_weight: u64,
    pub accepted: VoteRef<C>,
    pub confirmed: VoteRef<C>,
    pub prev: VoteRef<C>,
    pub slot_boundary: VoteRef<C>,
}
