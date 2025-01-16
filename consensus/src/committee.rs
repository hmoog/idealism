use std::collections::HashMap;
use std::sync::Arc;
use utils::ArcKey;
use crate::committee_member::CommitteeMember;
use crate::committee_member_id::CommitteeMemberID;
use crate::config::Config;
use crate::error::Error;
use crate::votes_by_issuer::VotesByIssuer;

pub struct Committee<T: Config>(Arc<CommitteeData<T>>);

struct CommitteeData<T: Config> {
    members_by_id: Arc<HashMap<ArcKey<T>, Arc<CommitteeMember<T::CommitteeMemberID>>>>,
    total_weight: u64,
    online_weight: u64,
}

impl <T> Clone for CommitteeData<T> where T: Config {
    fn clone(&self) -> Self {
        Self {
            members_by_id: self.members_by_id.clone(),
            total_weight: self.total_weight,
            online_weight: self.online_weight,
        }
    }
}

impl <T: Config> Committee<T> {
    pub fn iter(&self) -> impl Iterator<Item = &CommitteeMember<T>> {
        self.0.members_by_id.values().map(|member| &**member)
    }

    pub fn byzantine_threshold_for_acceptance(&self) -> u64 {
        self.online_weight() / 3
    }

    pub fn byzantine_threshold_for_confirmation(&self) -> u64 {
        self.total_weight() / 3
    }

    pub fn acceptance_threshold(&self) -> u64 {
        self.online_weight() - self.byzantine_threshold_for_acceptance()
    }

    pub fn confirmation_threshold(&self) -> u64 {
        self.total_weight() - self.byzantine_threshold_for_confirmation()
    }

    pub fn referenced_round_weight(&self, votes: &VotesByIssuer<T>) -> Result<u64, Error> {
        let mut latest_round = 0;
        let mut referenced_round_weight = 0;

        for (issuer, votes) in votes {
            if let Some(member) = self.0.members_by_id.get(issuer) {
                if let Some(vote_ref) = votes.first() {
                    let vote = vote_ref.as_vote()?;
                    if vote.round() > latest_round {
                        latest_round = vote.round();
                        referenced_round_weight = member.weight();
                    } else if vote.round() == latest_round {
                        referenced_round_weight += member.weight();
                    }
                }
            }
        }

        Ok(referenced_round_weight)
    }

    pub fn total_weight(&self) -> u64 {
        self.0.total_weight
    }

    pub fn online_weight(&self) -> u64 {
        self.0.online_weight
    }

    pub fn member(&self, member_id: &ArcKey<T>) -> Option<&CommitteeMember<T>> {
        self.0.members_by_id.get(member_id).map(|member| &**member)
    }

    pub fn member_weight(&self, member_id: &ArcKey<T>) -> u64 {
        self.0.members_by_id.get(member_id).map(|member| member.weight()).unwrap_or(0)
    }

    pub fn is_member_online(&self, member_id: &ArcKey<T>) -> bool {
        self.0.members_by_id.get(member_id).map_or(false, |member| member.is_online())
    }

    pub fn set_online(&self, member_id: &ArcKey<T>, online: bool) -> Self {
        let mut new_committee = Committee(self.0.clone());

        if let Some(member) = self.0.members_by_id.get(member_id) {
            if member.is_online() != online {
                // Create a mutable clone of the CommitteeData if necessary
                let data = Arc::make_mut(&mut new_committee.0);

                // Update online weight
                if online {
                    data.online_weight += member.weight();
                } else {
                    data.online_weight -= member.weight();
                }

                // Make the HashMap mutable, then make the member mutable, and update its state
                Arc::make_mut(
                    Arc::make_mut(&mut data.members_by_id)
                        .get_mut(member_id)
                        .expect("member must exist")
                ).set_online(online);
            }
        }

        new_committee
    }
}

impl<ID: CommitteeMemberID, T: IntoIterator<Item = CommitteeMember<ID>>> From<T> for Committee<ID> {
    fn from(members: T) -> Self {
        let (members_by_id, weight, online_weight) = members.into_iter().fold(
            (HashMap::new(), 0, 0),
            |(mut map, total_weight, online_weight), member| {
                let member_weight = member.weight();
                let updated_weight = total_weight + member_weight;
                let updated_online_weight = if member.is_online() {
                    online_weight + member_weight
                } else {
                    online_weight
                };
                map.insert(member.key().clone(), Arc::new(member));
                (map, updated_weight, updated_online_weight)
            },
        );

        Committee(Arc::new(CommitteeData {
            members_by_id: Arc::new(members_by_id),
            total_weight: weight,
            online_weight,
        }))
    }
}


impl<T: Config> Clone for Committee<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::committee_member::CommitteeMember;

    #[test]
    fn test_committee() {
        let committee = Committee::from(vec![
            CommitteeMember::new(1).with_weight(10).with_online(true),
            CommitteeMember::new(2).with_weight(20).with_online(false)
        ]);

        // assert initial state
        assert_eq!(committee.total_weight(), 30);
        assert_eq!(committee.online_weight(), 10);
        assert!(committee.is_member_online(&ArcKey::new(1)));
        assert_eq!(committee.member_weight(&ArcKey::new(1)), 10);
        assert!(!committee.is_member_online(&ArcKey::new(2)));
        assert_eq!(committee.member_weight(&ArcKey::new(2)), 20);

        // set member 2 online
        let committee1 = committee.set_online(&ArcKey::new(2), true);
        assert_eq!(committee1.total_weight(), 30);
        assert_eq!(committee1.online_weight(), 30);
        assert!(committee1.is_member_online(&ArcKey::new(1)));
        assert_eq!(committee1.member_weight(&ArcKey::new(1)), 10);
        assert!(committee1.is_member_online(&ArcKey::new(2)));
        assert_eq!(committee1.member_weight(&ArcKey::new(2)), 20);

        // original committee is not changed
        assert_eq!(committee.total_weight(), 30);
        assert_eq!(committee.online_weight(), 10);
        assert!(committee.is_member_online(&ArcKey::new(1)));
        assert_eq!(committee.member_weight(&ArcKey::new(1)), 10);
        assert!(!committee.is_member_online(&ArcKey::new(2)));
        assert_eq!(committee.member_weight(&ArcKey::new(2)), 20);

        // set member 2 online again (no change / same underlying data)
        let committee2 = committee1.set_online(&ArcKey::new(2), true);
        assert!(Arc::ptr_eq(&committee1.0, &committee2.0));
    }
}