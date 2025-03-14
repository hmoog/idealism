use std::{collections::HashMap, sync::Arc};

use crate::{Member, MemberID};

pub(crate) struct Members {
    pub(crate) members_by_id: Arc<HashMap<MemberID, Arc<Member>>>,
    pub(crate) total_weight: u64,
    pub(crate) online_weight: u64,
}

impl Clone for Members {
    fn clone(&self) -> Self {
        Self {
            members_by_id: self.members_by_id.clone(),
            total_weight: self.total_weight,
            online_weight: self.online_weight,
        }
    }
}
