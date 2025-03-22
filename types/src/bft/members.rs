use std::{collections::HashMap, sync::Arc};

use crate::{
    bft::Member,
    hash::{Hashable, Hasher},
    ids::IssuerID,
};

pub struct Members {
    pub(crate) members_by_id: Arc<HashMap<IssuerID, Arc<Member>>>,
    pub(crate) total_weight: u64,
    pub(crate) online_weight: u64,
}

impl Hashable for Members {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        hasher.update(&postcard::to_allocvec(self).unwrap())
    }
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

mod serialization {
    use std::{collections::HashMap, fmt, sync::Arc};

    use serde::{
        Deserialize, Deserializer, Serialize, Serializer, de,
        de::{SeqAccess, Visitor},
        ser::SerializeSeq,
    };

    use crate::{
        bft::{Member, Members},
        ids::IssuerID,
    };

    impl Serialize for Members {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            // First serialize the members_by_id deterministically (sorted)
            let mut entries: Vec<(&IssuerID, &Arc<Member>)> = self.members_by_id.iter().collect();
            entries.sort_by(|a, b| a.0.cmp(b.0));

            // Serialize as a tuple: (sorted entries, total_weight, online_weight)
            let mut seq = serializer.serialize_seq(Some(3))?;

            // Serialize members_by_id as sequence of tuples
            seq.serialize_element(&entries)?;
            seq.serialize_element(&self.total_weight)?;
            seq.serialize_element(&self.online_weight)?;
            seq.end()
        }
    }
    impl<'de> Deserialize<'de> for Members {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_seq(MembersVisitor)
        }
    }

    struct MembersVisitor;

    impl<'de> Visitor<'de> for MembersVisitor {
        type Value = Members;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str(
                "Members serialized as (sorted members_by_id, total_weight, online_weight)",
            )
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            // Deserialize sorted entries
            let entries: Vec<(IssuerID, Arc<Member>)> = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(0, &self))?;

            // Reconstruct HashMap from entries
            let members_by_id: HashMap<IssuerID, Arc<Member>> = entries.into_iter().collect();

            // Deserialize total_weight
            let total_weight: u64 = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(1, &self))?;

            // Deserialize online_weight
            let online_weight: u64 = seq
                .next_element()?
                .ok_or_else(|| de::Error::invalid_length(2, &self))?;

            Ok(Members {
                members_by_id: Arc::new(members_by_id),
                total_weight,
                online_weight,
            })
        }
    }
}
