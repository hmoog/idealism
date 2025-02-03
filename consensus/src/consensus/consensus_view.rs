use crate::{
    ConfigInterface, Error, Vote, VoteBuilder, VotesByIssuer,
    consensus::consensus_mechanism::ConsensusMechanism,
};

pub struct ConsensusView<ID: ConfigInterface> {
    pub latest_confirmed_milestone: Vote<ID>,
    pub latest_accepted_milestone: Vote<ID>,
    pub heaviest_tip: Vote<ID>,
}

impl<ID: ConfigInterface> ConsensusView<ID> {
    pub fn from_vote_builder(src: &VoteBuilder<ID>) -> crate::Result<Self> {
        let mut consensus_mechanism = ConsensusMechanism::new(src.committee.clone());
        consensus_mechanism
            .scan_past_cone(VotesByIssuer::try_from(&src.votes_by_issuer)?.into())?;
        consensus_mechanism.scan_future_cone();

        Ok(Self {
            latest_confirmed_milestone: consensus_mechanism
                .last_accepted_milestone
                .ok_or(Error::NoAcceptedMilestoneInPastCone)?,
            latest_accepted_milestone: consensus_mechanism
                .last_confirmed_milestone
                .ok_or(Error::NoConfirmedMilestoneInPastCone)?,
            heaviest_tip: consensus_mechanism
                .heaviest_tip
                .expect("heaviest tip should be set"),
        })
    }
}

impl<ID: ConfigInterface> TryFrom<&VoteBuilder<ID>> for ConsensusView<ID> {
    type Error = Error;
    fn try_from(vote_builder: &VoteBuilder<ID>) -> crate::Result<Self> {
        ConsensusView::from_vote_builder(vote_builder)
    }
}
