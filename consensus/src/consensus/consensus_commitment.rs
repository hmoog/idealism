use std::{fmt::Debug, sync::Weak};

use crate::{ConfigInterface, VoteBuilder, VoteRef};

#[derive(Default)]
pub struct ConsensusCommitment<C: ConfigInterface> {
    pub milestone: VoteRef<C>,
    pub tip: VoteRef<C>,
}

impl<C: ConfigInterface> Debug for ConsensusCommitment<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConsensusCommitment")

            .field("accepted_milestone", &self.milestone)
            .field("heaviest_tip", &self.tip)
            .finish()
    }
}

impl<C: ConfigInterface> From<&Weak<VoteBuilder<C>>> for ConsensusCommitment<C> {
    fn from(src: &Weak<VoteBuilder<C>>) -> Self {
        Self {
            milestone: src.into(),
            tip: src.into(),
        }
    }
}

impl<C: ConfigInterface> Clone for ConsensusCommitment<C> {
    fn clone(&self) -> Self {
        Self {
            milestone: self.milestone.clone(),
            tip: self.tip.clone(),
        }
    }
}
