#[macro_export]
macro_rules! define_transparent {
    // Macro arm for newtypes with generics
    ($name:ident<$($generic:ident $(: $bound:tt)*),*>, $inner:ty) => {
        pub struct $name<$($generic $(: $bound)*),*>($inner);

        // Implement Clone
        impl<$($generic $(: $bound)*),*> Clone for $name<$($generic),*>
        where
            $inner: Clone,
        {
            fn clone(&self) -> Self {
                Self(self.0.clone())
            }
        }

        // Implement Deref
        impl<$($generic $(: $bound)*),*> std::ops::Deref for $name<$($generic),*> {
            type Target = $inner;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        // DerefMut
        impl<$($generic $(: $bound)*),*> std::ops::DerefMut for $name<$($generic),*> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };

    // Macro arm for newtypes without generics
    ($name:ident, $inner:ty) => {
        struct $name($inner);

        // Implement Clone
        impl Clone for $name
        where
            $inner: Clone,
        {
            fn clone(&self) -> Self {
                Self(self.0.clone())
            }
        }

        // Implement Deref
        impl std::ops::Deref for $name {
            type Target = $inner;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        // DerefMut
        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
    };
}
