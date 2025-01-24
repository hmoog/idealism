#[macro_export]
macro_rules! define_hashset {
    ($wrapper:ident, $item:ty, $generics:ident $(: $bounds:path)?) => {
        // Call the existing wrapper macro to create the basic structure
        $crate::define!($wrapper, std::collections::HashSet<$item>, $generics $(: $bounds)?);

        // Implement FromIterator for any type whose items can be converted into the inner type
        impl<$generics $(: $bounds)?, U> FromIterator<U> for $wrapper<$generics>
        where
            U: Into<$item>,
        {
            fn from_iter<I: IntoIterator<Item = U>>(iter: I) -> Self {
                Self(iter.into_iter().map(Into::into).collect())
            }
        }

        // Implement IntoIterator for owned wrapper
        impl<$generics $(: $bounds)?> IntoIterator for $wrapper<$generics> {
            type Item = $item;
            type IntoIter = std::collections::hash_set::IntoIter<$item>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.into_iter()
            }
        }

        // Implement IntoIterator for borrowed wrapper
        impl<'a, $generics $(: $bounds)?> IntoIterator for &'a $wrapper<$generics> {
            type Item = &'a $item;
            type IntoIter = std::collections::hash_set::Iter<'a, $item>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.iter()
            }
        }
    };
}