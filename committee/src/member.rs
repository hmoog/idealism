use types::ids::IssuerID;

pub struct Member {
    id: IssuerID,
    index: u64,
    weight: u64,
    online: bool,
}

impl Member {
    pub fn new(id: IssuerID) -> Self {
        Self {
            id,
            index: 0,
            weight: 1,
            online: true,
        }
    }

    pub fn with_index(mut self, index: u64) -> Self {
        self.index = index;
        self
    }

    pub fn with_weight(mut self, weight: u64) -> Self {
        self.weight = weight;
        self
    }

    pub fn with_online(mut self, online: bool) -> Self {
        self.online = online;
        self
    }

    pub fn id(&self) -> &IssuerID {
        &self.id
    }

    pub fn index(&self) -> u64 {
        self.index
    }

    pub fn weight(&self) -> u64 {
        self.weight
    }

    pub fn set_weight(&mut self, weight: u64) -> bool {
        if self.weight != weight {
            self.weight = weight;
            return true;
        }
        false
    }

    pub fn is_online(&self) -> bool {
        self.online
    }

    pub fn set_online(&mut self, online: bool) -> bool {
        if self.online != online {
            self.online = online;
            return true;
        }
        false
    }

    pub fn key(&self) -> &IssuerID {
        &self.id
    }
}

impl Clone for Member {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            index: self.index,
            weight: self.weight,
            online: self.online,
        }
    }
}

#[cfg(test)]
mod tests {
    use types::{
        hash::{Hashable, Hasher},
        ids::IssuerID,
    };

    use super::*;

    struct HashableID(i32);

    impl Hashable for HashableID {
        fn hash<H: Hasher>(&self, hasher: &mut H) {
            hasher.update(&self.0.to_be_bytes());
        }
    }

    #[test]
    fn test_new_committee_member() {
        let member_id = IssuerID::new(&HashableID(1337));
        let member = Member::new(member_id.clone());
        assert_eq!(*member.id(), member_id);
        assert_eq!(member.weight(), 1);
        assert!(member.is_online());
    }

    #[test]
    fn test_with_weight() {
        let member_id = IssuerID::new(&HashableID(1337));
        let member = Member::new(member_id.clone()).with_weight(10);
        assert_eq!(*member.id(), member_id);
        assert_eq!(member.weight(), 10);
        assert!(member.is_online());
    }

    #[test]
    fn test_with_online() {
        let member_id = IssuerID::new(&HashableID(1337));
        let member = Member::new(member_id.clone()).with_online(false);
        assert_eq!(*member.id(), member_id);
        assert_eq!(member.weight(), 1);
        assert!(!member.is_online());
    }

    #[test]
    fn test_set_weight() {
        let member_id = IssuerID::new(&HashableID(1337));
        let mut member = Member::new(member_id.clone());
        assert!(member.set_weight(10));
        assert_eq!(member.weight(), 10);
        assert!(!member.set_weight(10));
    }

    #[test]
    fn test_set_online() {
        let member_id = IssuerID::new(&HashableID(1337));
        let mut member = Member::new(member_id);
        assert!(member.set_online(false));
        assert!(!member.is_online());
        assert!(!member.set_online(false));
    }
}
