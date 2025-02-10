use crate::{ConfigInterface, VoteRef};

pub struct Milestone<T: ConfigInterface> {
    pub round_weight: u64,
    pub accepted: VoteRef<T>,
    pub confirmed: VoteRef<T>,
    pub prev: VoteRef<T>,
}
