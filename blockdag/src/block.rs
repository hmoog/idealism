use std::{fmt::Debug, hash::Hash};

pub trait Block: Sync + Send + Debug + 'static {
    type ID: Sync + Send + Hash + Eq + Clone + 'static;

    fn id(&self) -> &Self::ID;

    fn parents(&self) -> &[Self::ID];
}
