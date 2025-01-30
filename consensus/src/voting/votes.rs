use newtype::{Clone0, Default0, Deref0, FromIterator0, IntoIterator0};

use crate::{ConfigInterface, Vote, utils::max_set::MaxSet};

#[derive(Clone0, Default0, Deref0, FromIterator0, IntoIterator0)]
pub struct Votes<Config: ConfigInterface>(MaxSet<Vote<Config>>);

impl<Config: ConfigInterface> Votes<Config> {
    pub fn round(&self) -> u64 {
        self.heaviest_element().map_or(0, |v| v.round)
    }
}
