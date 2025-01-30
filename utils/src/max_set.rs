use std::{collections::HashSet, hash::Hash};

/// HashSet variant that tracks its maximum element during modifications.
///
/// # Type Parameters
/// - `E`: Element type that can be hashed, compared and cloned
pub struct MaxSet<E: Element> {
    elements: HashSet<E>,
    heaviest_element: Option<E>,
}

impl<E: Element> MaxSet<E> {
    /// Inserts an element into the set.
    ///
    /// # Returns
    /// `true` if the element was not present in the set.
    ///
    /// # Safety
    /// Updates the tracked maximum element if the new element is greater.
    pub fn insert(&mut self, element: E) -> bool {
        if self
            .heaviest_element
            .as_ref()
            .map_or(true, |v| element > *v)
        {
            self.heaviest_element = Some(element.clone());
        }
        self.elements.insert(element)
    }

    /// Returns a reference to the maximum element if the set is non-empty.
    pub fn heaviest_element(&self) -> Option<&E> {
        self.heaviest_element.as_ref()
    }

    /// Removes all elements from the set.
    pub fn clear(&mut self) {
        self.heaviest_element = None;
        self.elements.clear();
    }
}

impl<E: Element> Default for MaxSet<E> {
    fn default() -> Self {
        Self {
            elements: HashSet::new(),
            heaviest_element: None,
        }
    }
}

impl<E: Element, I: Into<E>> FromIterator<I> for MaxSet<E> {
    fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
        let mut result = Self::default();
        result.extend(iter.into_iter().map(Into::into));
        result
    }
}

impl<E: Element, I: Into<E>> Extend<I> for MaxSet<E> {
    fn extend<T: IntoIterator<Item = I>>(&mut self, iter: T) {
        iter.into_iter().for_each(|v| {
            self.insert(v.into());
        });
    }
}

impl<E: Element> IntoIterator for MaxSet<E> {
    type Item = E;
    type IntoIter = std::collections::hash_set::IntoIter<E>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.into_iter()
    }
}

impl<'a, E: Element> IntoIterator for &'a MaxSet<E> {
    type Item = &'a E;
    type IntoIter = std::collections::hash_set::Iter<'a, E>;

    fn into_iter(self) -> Self::IntoIter {
        self.elements.iter()
    }
}

impl<E: Element> Clone for MaxSet<E> {
    fn clone(&self) -> Self {
        Self {
            elements: self.elements.clone(),
            heaviest_element: self.heaviest_element.clone(),
        }
    }
}

/// Trait bound for elements in [`MaxSet`] requiring equality, hashing, ordering and cloning
/// capabilities.
pub trait Element: Eq + Hash + Ord + Clone {}

impl<T: Eq + Hash + Ord + Clone> Element for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_heaviest() {
        let mut set = MaxSet::default();
        assert_eq!(set.heaviest_element(), None);

        set.insert(1);
        assert_eq!(set.heaviest_element(), Some(&1));

        set.insert(5);
        assert_eq!(set.heaviest_element(), Some(&5));

        set.insert(3);
        assert_eq!(set.heaviest_element(), Some(&5));
    }

    #[test]
    fn test_clear() {
        let mut set = MaxSet::default();
        set.insert(1);
        set.insert(2);
        set.clear();
        assert_eq!(set.heaviest_element(), None);
        assert!(set.into_iter().next().is_none());
    }

    #[test]
    fn test_from_iterator() {
        let set: MaxSet<i32> = vec![1, 3, 2].into_iter().collect();
        assert_eq!(set.heaviest_element(), Some(&3));
    }

    #[test]
    fn test_extend() {
        let mut set = MaxSet::default();
        set.insert(1);
        set.extend(vec![2, 4, 3]);
        assert_eq!(set.heaviest_element(), Some(&4));
    }

    #[test]
    fn test_duplicate_insert() {
        let mut set = MaxSet::default();
        assert!(set.insert(1));
        assert!(!set.insert(1));
        assert_eq!(set.heaviest_element(), Some(&1));
    }

    #[test]
    fn test_clone() {
        let mut set = MaxSet::default();
        set.insert(1);
        set.insert(2);

        let cloned = set.clone();
        assert_eq!(cloned.heaviest_element(), Some(&2));
        assert!(cloned.into_iter().collect::<Vec<_>>().contains(&1));
    }
}
