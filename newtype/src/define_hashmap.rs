#[macro_export]
macro_rules! define_hashmap {
    ($wrapper:ident, $key:ty, $value:ty, $generics:ident $(: $bounds:path)?) => {
        // Call the existing wrapper macro to create the basic structure
        $crate::define!($wrapper, std::collections::HashMap<$key, $value>, $generics $(: $bounds)?);

        // Implement FromIterator for collections of tuples
        impl<$generics $(: $bounds)?> FromIterator<($key, $value)> for $wrapper<$generics> {
            fn from_iter<I: IntoIterator<Item = ($key, $value)>>(
                iter: I,
            ) -> Self {
                Self(iter.into_iter().collect())
            }
        }

        // Implement FromIterator for borrowed tuples
        impl<'a, $generics $(: $bounds)?> FromIterator<(&'a $key, &'a $value)> for $wrapper<$generics>
        where
            $key: Clone,
            $value: Clone,
        {
            fn from_iter<I: IntoIterator<Item = (&'a $key, &'a $value)>>(
                iter: I,
            ) -> Self {
                iter.into_iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect()
            }
        }

        // Implement IntoIterator for owned wrapper
        impl<$generics $(: $bounds)?> IntoIterator for $wrapper<$generics> {
            type Item = ($key, $value);
            type IntoIter = std::collections::hash_map::IntoIter<$key, $value>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.into_iter()
            }
        }

        // Implement IntoIterator for borrowed wrapper
        impl<'a, $generics $(: $bounds)?> IntoIterator for &'a $wrapper<$generics> {
            type Item = (&'a $key, &'a $value);
            type IntoIter = std::collections::hash_map::Iter<'a, $key, $value>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.iter()
            }
        }

        // Implement IntoIterator for mutable borrowed wrapper
        impl<'a, $generics $(: $bounds)?> IntoIterator for &'a mut $wrapper<$generics> {
            type Item = (&'a $key, &'a mut $value);
            type IntoIter = std::collections::hash_map::IterMut<'a, $key, $value>;

            fn into_iter(self) -> Self::IntoIter {
                self.0.iter_mut()
            }
        }
    };
}
