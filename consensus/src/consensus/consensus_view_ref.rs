use std::sync::Weak;

use crate::{ConfigInterface, ConsensusView, VoteBuilder, VoteRef};

#[derive(Default)]
pub struct ConsensusViewRef<ID: ConfigInterface> {
    pub latest_confirmed_milestone: VoteRef<ID>,
    pub latest_accepted_milestone: VoteRef<ID>,
    pub heaviest_tip: VoteRef<ID>,
}

impl<ID: ConfigInterface> ConsensusViewRef<ID> {
    pub fn from_consensus_view(src: ConsensusView<ID>) -> Self {
        Self {
            latest_confirmed_milestone: src.latest_confirmed_milestone.into(),
            latest_accepted_milestone: src.latest_accepted_milestone.into(),
            heaviest_tip: src.heaviest_tip.into(),
        }
    }
}

impl<ID: ConfigInterface> From<ConsensusView<ID>> for ConsensusViewRef<ID> {
    fn from(src: ConsensusView<ID>) -> Self {
        Self::from_consensus_view(src)
    }
}

impl<ID: ConfigInterface> From<&Weak<VoteBuilder<ID>>> for ConsensusViewRef<ID> {
    fn from(src: &Weak<VoteBuilder<ID>>) -> Self {
        Self {
            latest_confirmed_milestone: src.clone().into(),
            latest_accepted_milestone: src.into(),
            heaviest_tip: src.into(),
        }
    }
}

impl<ID: ConfigInterface> Clone for ConsensusViewRef<ID> {
    fn clone(&self) -> Self {
        Self {
            latest_confirmed_milestone: self.latest_confirmed_milestone.clone(),
            latest_accepted_milestone: self.latest_accepted_milestone.clone(),
            heaviest_tip: self.heaviest_tip.clone(),
        }
    }
}
