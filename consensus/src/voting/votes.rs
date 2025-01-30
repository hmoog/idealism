use newtype::{Clone0, Default0, Deref0, FromIterator0, IntoIterator0};
use utils::MaxSet;

use crate::{ConfigInterface, Vote};

#[derive(Clone0, Default0, Deref0, FromIterator0, IntoIterator0)]
pub struct Votes<Config: ConfigInterface>(MaxSet<Vote<Config>>);

impl<Config: ConfigInterface> Votes<Config> {
    pub fn round(&self) -> u64 {
        self.heaviest_element().map_or(0, |v| v.round)
    }
}
