use utils::ArcKey;

use crate::CommitteeMemberID;

pub struct CommitteeMember<T: CommitteeMemberID> {
    id: ArcKey<T>,
    index: u64,
    weight: u64,
    online: bool,
}

impl<T: CommitteeMemberID> CommitteeMember<T> {
    pub fn new(id: T) -> Self {
        Self {
            id: ArcKey::new(id),
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

    pub fn id(&self) -> &T {
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

    pub(crate) fn key(&self) -> &ArcKey<T> {
        &self.id
    }
}

impl<T: CommitteeMemberID> Clone for CommitteeMember<T> {
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
    use super::*;

    #[test]
    fn test_new_committee_member() {
        let member = CommitteeMember::new(1337);
        assert_eq!(*member.id(), 1337);
        assert_eq!(member.weight(), 1);
        assert!(member.is_online());
    }

    #[test]
    fn test_with_weight() {
        let member = CommitteeMember::new(1337).with_weight(10);
        assert_eq!(*member.id(), 1337);
        assert_eq!(member.weight(), 10);
        assert!(member.is_online());
    }

    #[test]
    fn test_with_online() {
        let member = CommitteeMember::new(1337).with_online(false);
        assert_eq!(*member.id(), 1337);
        assert_eq!(member.weight(), 1);
        assert!(!member.is_online());
    }

    #[test]
    fn test_set_weight() {
        let mut member = CommitteeMember::new(1337);
        assert!(member.set_weight(10));
        assert_eq!(member.weight(), 10);
        assert!(!member.set_weight(10));
    }

    #[test]
    fn test_set_online() {
        let mut member = CommitteeMember::new(1337);
        assert!(member.set_online(false));
        assert!(!member.is_online());
        assert!(!member.set_online(false));
    }
}
