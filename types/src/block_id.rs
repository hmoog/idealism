use crate::hasher::blake2b;
use crate::id::Id;

pub type BlockID = Id<blake2b::Hasher>;