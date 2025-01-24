#[macro_export]
macro_rules! define {
    ($wrapper:ident, $inner:ty $(, $generics:ident $(: $bounds:path)? )*) => {
        pub struct $wrapper<$($generics $(: $bounds)?),*>(pub $inner);

        impl<$($generics $(: $bounds)?),*> $wrapper<$($generics),*> {
            pub fn new(inner: $inner) -> Self {
                Self(inner)
            }

            pub fn into_inner(self) -> $inner {
                self.0
            }

            pub fn as_inner(&self) -> &$inner {
                &self.0
            }

            pub fn as_inner_mut(&mut self) -> &mut $inner {
                &mut self.0
            }
        }

        // Implement Clone for the wrapper type if the inner type is Clone
        impl<$($generics $(: $bounds)?),*> Clone for $wrapper<$($generics),*>
        where
            $inner: Clone
        {
            fn clone(&self) -> Self {
                $wrapper(self.0.clone())
            }
        }

        // Implement Default for the wrapper type if the inner type is Default
        impl<$($generics $(: $bounds)?),*> Default for $wrapper<$($generics),*>
        where
            $inner: Default
        {
            fn default() -> Self {
                $wrapper(<$inner>::default())
            }
        }

        // Optional: Implement Deref for ergonomic access
        impl<$($generics $(: $bounds)?),*> std::ops::Deref for $wrapper<$($generics),*> {
            type Target = $inner;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        // Optional: Implement DerefMut for mutable access
        impl<$($generics $(: $bounds)?),*> std::ops::DerefMut for $wrapper<$($generics),*> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}