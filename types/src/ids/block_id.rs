use crate::{hash::Blake2bHasher, ids::Id};

pub type BlockID = Id<Blake2bHasher>;
