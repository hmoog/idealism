use crate::{Config, VoteRef};

pub struct Milestone<T: Config> {
    pub leader_weight: u64,
    pub accepted: VoteRef<T>,
    pub confirmed: VoteRef<T>,
    pub prev: VoteRef<T>,
}
