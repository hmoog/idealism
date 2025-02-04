use std::{fmt::Debug, sync::Weak};

use crate::{ConfigInterface, VoteBuilder, VoteRef};

#[derive(Default)]
pub struct ConsensusCommitment<C: ConfigInterface> {
    pub confirmed_milestone: VoteRef<C>,
    pub accepted_milestone: VoteRef<C>,
    pub heaviest_tip: VoteRef<C>,
}

impl<C: ConfigInterface> Debug for ConsensusCommitment<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConsensusCommitment")
            .field("confirmed_milestone", &self.confirmed_milestone)
            .field("accepted_milestone", &self.accepted_milestone)
            .field("heaviest_tip", &self.heaviest_tip)
            .finish()
    }
}

impl<C: ConfigInterface> From<&Weak<VoteBuilder<C>>> for ConsensusCommitment<C> {
    fn from(src: &Weak<VoteBuilder<C>>) -> Self {
        Self {
            confirmed_milestone: src.into(),
            accepted_milestone: src.into(),
            heaviest_tip: src.into(),
        }
    }
}

impl<C: ConfigInterface> Clone for ConsensusCommitment<C> {
    fn clone(&self) -> Self {
        Self {
            confirmed_milestone: self.confirmed_milestone.clone(),
            accepted_milestone: self.accepted_milestone.clone(),
            heaviest_tip: self.heaviest_tip.clone(),
        }
    }
}
