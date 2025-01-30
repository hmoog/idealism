use newtype::{CloneInner, DefaultInner, DerefInner, FromIteratorInner, IntoIteratorInner};

use crate::{ConfigInterface, Vote, utils::set::Set};

#[derive(IntoIteratorInner, FromIteratorInner, DefaultInner, DerefInner, CloneInner)]
pub struct Votes<Config: ConfigInterface>(Set<Vote<Config>>);

impl<Config: ConfigInterface> Votes<Config> {
    pub fn round(&self) -> u64 {
        self.heaviest_element().map_or(0, |v| v.round)
    }
}
