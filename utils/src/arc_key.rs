use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;

pub struct ArcKey<T>(Arc<T>);

impl<T> ArcKey<T> {
    pub fn new(value: T) -> Self {
        ArcKey(Arc::new(value))
    }
}

impl<T> Clone for ArcKey<T> {
    fn clone(&self) -> Self {
        ArcKey(Arc::clone(&self.0))
    }
}

impl<T: Hash> Hash for ArcKey<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl<T> Deref for ArcKey<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: PartialEq> PartialEq for ArcKey<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || *self.0 == *other.0
    }
}

impl<T: Eq> Eq for ArcKey<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_arc_key_behaves_correctly() {
        let id1 = ArcKey::new(1);
        let id2 = ArcKey::new(1); // Same content, different Arc
        let id3 = id1.clone(); // Points to the same Arc as id1

        let mut map: HashMap<ArcKey<usize>, String> = HashMap::new();
        map.insert(id1, "Hello".to_string());

        // id2 has the same content as id1, so it should be treated as the same key
        assert!(map.contains_key(&id2));

        // id3 points to the same Arc as id1, so it should be treated as the same key
        assert!(map.contains_key(&id3));

        // Verify the value associated with the key
        assert_eq!(map.get(&id2), Some(&"Hello".to_string()));
    }
}