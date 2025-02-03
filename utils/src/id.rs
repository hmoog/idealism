use std::{
    fmt::Debug,
    hash::{Hash, Hasher},
    ops::Deref,
    sync::Arc,
};

pub struct Id<T>(Arc<T>);

impl<T> Id<T> {
    pub fn new(value: T) -> Self {
        Id(Arc::new(value))
    }
}

impl<T: Debug> Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl<T> Clone for Id<T> {
    fn clone(&self) -> Self {
        Id(Arc::clone(&self.0))
    }
}

impl<T: Hash> Hash for Id<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T> Deref for Id<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: PartialEq> PartialEq for Id<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || *self.0 == *other.0
    }
}

impl<T: Eq> Eq for Id<T> {}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_arc_key_behaves_correctly() {
        let id1 = Id::new(1);
        let id2 = Id::new(1); // Same content, different Arc
        let id3 = id1.clone(); // Points to the same Arc as id1

        let mut map: HashMap<Id<usize>, String> = HashMap::new();
        map.insert(id1, "Hello".to_string());

        // id2 has the same content as id1, so it should be treated as the same key
        assert!(map.contains_key(&id2));

        // id3 points to the same Arc as id1, so it should be treated as the same key
        assert!(map.contains_key(&id3));

        // Verify the value associated with the key
        assert_eq!(map.get(&id2), Some(&"Hello".to_string()));
    }
}
