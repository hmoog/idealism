use crate::{hasher::blake2b, id::Id};

pub type BlockID = Id<blake2b::Hasher>;
