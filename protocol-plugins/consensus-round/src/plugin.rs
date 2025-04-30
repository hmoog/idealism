use std::{
    collections::HashSet,
    sync::{Arc, Mutex, Weak},
};

use block_dag::BlockDAG;
use common::{
    bft::Member,
    blocks::BlockMetadata,
    errors::Result,
    ids::IssuerID,
    rx::{
        Callbacks, Subscription,
        UpdateType::{Notify, Retain},
        Variable,
    },
};
use consensus::Consensus;
use protocol::{ManagedPlugin, Plugins};
use virtual_voting::{Issuer, VirtualVotingConfig, Vote};

pub struct ConsensusRound<C: VirtualVotingConfig> {
    pub started: Variable<u64>,
    pub completed: Variable<u64>,
    pub seen_participants: Variable<HashSet<IssuerID>>,
    pub seen_weight: Variable<u64>,
    block_dag_subscription: Mutex<Option<BlockDAGSubscription>>,
    consensus_subscription: Mutex<Option<ConsensusSubscription<C>>>,
    consensus: Arc<Consensus<C>>,
}

impl<C: VirtualVotingConfig> ConsensusRound<C> {
    fn new(weak: &Weak<Self>, plugins: &mut Plugins) -> Self {
        let consensus: Arc<Consensus<C>> = plugins.load();

        Self {
            started: Default::default(),
            completed: Default::default(),
            seen_participants: Default::default(),
            seen_weight: Default::default(),
            block_dag_subscription: Mutex::new(Some(Self::block_dag_subscription(
                &plugins.load(),
                weak.clone(),
            ))),
            consensus_subscription: Mutex::new(Some(Self::consensus_subscription(
                &consensus,
                weak.clone(),
            ))),
            consensus,
        }
    }

    fn shutdown(&self) {
        self.block_dag_subscription.lock().unwrap().take();
        self.consensus_subscription.lock().unwrap().take();
    }

    fn block_dag_subscription(block_dag: &Arc<BlockDAG>, weak: Weak<Self>) -> BlockDAGSubscription {
        block_dag.subscribe(move |block| {
            let weak = weak.clone();

            block.metadata::<Arc<Vote<C>>>().attach(move |vote| {
                if let Some(consensus_round) = weak.upgrade() {
                    if let Err(err) = consensus_round.process_vote(vote) {
                        // TODO: handle the error more elegantly
                        println!("{:?}", err);
                    }
                }
            });
        })
    }

    fn consensus_subscription(
        consensus: &Arc<Consensus<C>>,
        weak: Weak<Self>,
    ) -> ConsensusSubscription<C> {
        consensus.heaviest_milestone_vote.subscribe({
            move |(_, new)| {
                if let Some(new) = new {
                    if let Some(consensus_round) = weak.upgrade() {
                        consensus_round.update_started(new.round);
                    }
                }
            }
        })
    }

    fn process_vote(&self, vote: &Vote<C>) -> Result<()> {
        if vote.milestone.is_none() {
            return Ok(());
        }

        self.started.must_read(|round| {
            if vote.round != *round {
                return;
            }

            self.consensus.committee.must_read(|committee| {
                let (threshold, _) = committee.consensus_threshold();

                match &vote.issuer {
                    Issuer::User(issuer) => {
                        if let Some(member) = committee.member(issuer) {
                            self.update_seen_participants(vote.round, member, threshold);
                        }
                    }
                    Issuer::Genesis => {
                        for (issuer, _) in &vote.referenced_milestones {
                            if let Some(member) = committee.member(issuer) {
                                self.update_seen_participants(vote.round, member, threshold);
                            }
                        }
                    }
                };
            });
        });

        Ok(())
    }

    fn update_started(&self, new: u64) {
        self.started
            .compute::<(), _>(|old| match old {
                Some(old) if old >= new => Retain(Some(old)),
                _ => {
                    self.seen_participants.set(HashSet::new());
                    self.seen_weight.set(0);

                    Notify(old, Some(new))
                }
            })
            .expect("must not fail");
    }

    fn update_seen_participants(&self, round: u64, member: &Member, threshold: u64) {
        self.seen_participants
            .compute::<(), _>(|participants| {
                let mut participants = participants.unwrap_or_default();
                if participants.insert(member.id().clone()) {
                    self.update_seen_weight(round, member.weight(), threshold);
                }

                Notify(None, Some(participants))
            })
            .expect("seen_participants.compute should never fail");
    }

    fn update_seen_weight(&self, round: u64, weight: u64, threshold: u64) {
        let result = self.seen_weight.compute::<(), _>(|old| {
            let new = old.unwrap_or(0) + weight;
            if new > threshold {
                self.completed.track_max(round);
            }

            Notify(old, Some(new))
        });
        result.expect("seen_weight.compute should never fail");
    }
}

impl<C: VirtualVotingConfig> ManagedPlugin for ConsensusRound<C> {
    fn construct(plugins: &mut Plugins) -> Arc<Self> {
        Arc::new_cyclic(|weak| Self::new(weak, plugins))
    }

    fn shutdown(&self) {
        self.shutdown();
    }
}

type BlockDAGSubscription = Subscription<Callbacks<BlockMetadata>>;
type ConsensusSubscription<C> = Subscription<Callbacks<(Option<Vote<C>>, Option<Vote<C>>)>>;
