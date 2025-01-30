use std::{collections::HashMap, sync::Arc};

use utils::Id;

use crate::{CommitteeMember, ConfigInterface};

pub(crate) struct CommitteeData<C: ConfigInterface> {
    pub(crate) members_by_id: Arc<CommitteeMembersByID<C>>,
    pub(crate) total_weight: u64,
    pub(crate) online_weight: u64,
}

impl<T> Clone for CommitteeData<T>
where
    T: ConfigInterface,
{
    fn clone(&self) -> Self {
        Self {
            members_by_id: self.members_by_id.clone(),
            total_weight: self.total_weight,
            online_weight: self.online_weight,
        }
    }
}

type CommitteeMembersByID<C> = HashMap<
    Id<<C as ConfigInterface>::IssuerID>,
    Arc<CommitteeMember<<C as ConfigInterface>::IssuerID>>,
>;
