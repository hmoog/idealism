use std::{collections::HashMap, sync::Arc};

use utils::Id;

use crate::{Member, MemberID};

pub(crate) struct Members<C: MemberID> {
    pub(crate) members_by_id: Arc<HashMap<Id<C>, Arc<Member<C>>>>,
    pub(crate) total_weight: u64,
    pub(crate) online_weight: u64,
}

impl<T: MemberID> Clone for Members<T> {
    fn clone(&self) -> Self {
        Self {
            members_by_id: self.members_by_id.clone(),
            total_weight: self.total_weight,
            online_weight: self.online_weight,
        }
    }
}
