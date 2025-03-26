use std::sync::Arc;

use types::{bft::Committee, rx::Variable};
use virtual_voting::{Config, Vote};

pub struct State<C: Config> {
    pub chain_index: Variable<u64>,
    pub heaviest_milestone: Variable<Vote<C>>,
    pub round: Arc<Variable<u64>>,
    pub committee: Arc<Variable<Committee>>,
}

impl<C: Config> State<C> {
    pub fn new() -> Self {
        let state = Self::default();

        state
            .heaviest_milestone
            .subscribe({
                let round = state.round.clone();
                let committee = state.committee.clone();

                move |update| {
                    if let Some(heaviest_milestone) = &update.1 {
                        round.track_max(heaviest_milestone.round);

                        committee
                            .set_if_none_or(heaviest_milestone.committee.clone(), |old, new| {
                                new.commitment() != old.commitment()
                            });
                    }
                }
            })
            .forever();

        state
    }
}

impl<C: Config> Default for State<C> {
    fn default() -> Self {
        Self {
            chain_index: Variable::new(),
            heaviest_milestone: Variable::new(),
            round: Arc::new(Variable::new()),
            committee: Arc::new(Variable::new()),
        }
    }
}
