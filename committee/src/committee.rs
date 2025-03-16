use std::{collections::HashMap, sync::Arc};

use types::ids::IssuerID;

use crate::{Member, Members};

pub struct Committee(Arc<Members>);

impl Committee {
    pub fn total_weight(&self) -> u64 {
        self.0.total_weight
    }

    pub fn online_weight(&self) -> u64 {
        self.0.online_weight
    }

    pub fn member(&self, member_id: &IssuerID) -> Option<&Member> {
        self.0.members_by_id.get(member_id).map(|member| &**member)
    }

    pub fn members(&self) -> Vec<Member> {
        let mut values: Vec<_> = self
            .0
            .members_by_id
            .values()
            .map(|member| Member::clone(member))
            .collect();
        values.sort_by_key(|item| item.index());
        values
    }

    pub fn member_weight(&self, member_id: &IssuerID) -> u64 {
        self.0
            .members_by_id
            .get(member_id)
            .map(|member| member.weight())
            .unwrap_or(0)
    }

    pub fn is_member_online(&self, member_id: &IssuerID) -> bool {
        self.0
            .members_by_id
            .get(member_id)
            .is_some_and(|m| m.is_online())
    }

    pub fn set_online(&self, member_id: &IssuerID, online: bool) -> Self {
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
                        .expect("member must exist"),
                )
                .set_online(online);
            }
        }

        new_committee
    }

    pub fn iter(&self) -> impl Iterator<Item = &Member> {
        self.0.members_by_id.values().map(|member| &**member)
    }

    pub fn size(&self) -> u64 {
        self.0.members_by_id.len() as u64
    }
}

impl<T: IntoIterator<Item = Member>> From<T> for Committee {
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

        Committee(Arc::new(Members {
            members_by_id: Arc::new(members_by_id),
            total_weight,
            online_weight,
        }))
    }
}

impl Clone for Committee {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
