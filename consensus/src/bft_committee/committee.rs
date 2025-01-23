use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Arc;
use utils::ArcKey;
use crate::bft_committee::CommitteeData;
use crate::bft_committee::CommitteeMember;
use crate::ConfigInterface;
use crate::error::Error;
use crate::VoteRefsByIssuer;

pub struct Committee<C: ConfigInterface>(Arc<CommitteeData<C>>);

impl <T: ConfigInterface> Committee<T> {
    pub fn acceptance_threshold(&self) -> u64 {
        self.online_weight() - self.online_weight() / 3
    }

    pub fn confirmation_threshold(&self) -> u64 {
        self.total_weight() - self.total_weight() / 3
    }

    pub fn referenced_round_weight(&self, votes: &VoteRefsByIssuer<T>) -> Result<u64, Error> {
        let mut latest_round = 0;
        let mut referenced_round_weight = 0;

        for (issuer, votes) in votes {
            if let Some(member) = self.0.members_by_id.get(issuer) {
                if let Some(vote_ref) = votes.first() {
                    let vote = vote_ref.as_vote()?;
                    match vote.round().cmp(&latest_round) {
                        Ordering::Greater => {
                            latest_round = vote.round();
                            referenced_round_weight = member.weight();
                        }
                        Ordering::Equal => {
                            referenced_round_weight += member.weight();
                        }
                        Ordering::Less => continue,
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

    pub fn member(&self, member_id: &ArcKey<T::CommitteeMemberID>) -> Option<&CommitteeMember<T::CommitteeMemberID>> {
        self.0.members_by_id.get(member_id).map(|member| &**member)
    }

    pub fn members(&self) -> Vec<CommitteeMember<T::CommitteeMemberID>> {
        let mut values: Vec<_> = self.0.members_by_id.values().map(|member| CommitteeMember::clone(member)).collect();
        values.sort_by_key(|item| item.index());
        values
    }

    pub fn member_weight(&self, member_id: &ArcKey<T::CommitteeMemberID>) -> u64 {
        self.0.members_by_id.get(member_id).map(|member| member.weight()).unwrap_or(0)
    }

    pub fn is_member_online(&self, member_id: &ArcKey<T::CommitteeMemberID>) -> bool {
        self.0.members_by_id.get(member_id).map_or(false, |member| member.is_online())
    }

    pub fn set_online(&self, member_id: &ArcKey<T::CommitteeMemberID>, online: bool) -> Self {
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

    pub fn iter(&self) -> impl Iterator<Item = &CommitteeMember<T::CommitteeMemberID>> {
        self.0.members_by_id.values().map(|member| &**member)
    }

    pub fn size(&self) -> u64 {
        self.0.members_by_id.len() as u64
    }
}

impl<C: ConfigInterface, T: IntoIterator<Item = CommitteeMember<C::CommitteeMemberID>>> From<T> for Committee<C> {
    fn from(members: T) -> Self {
        let (members_by_id, total_weight, online_weight) = members.into_iter().fold(
            (HashMap::new(), 0, 0),
            |(mut map, total_weight, online_weight), member| {
                let member = member.with_index(map.len() as u64);
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
            total_weight,
            online_weight,
        }))
    }
}


impl<T: ConfigInterface> Clone for Committee<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
