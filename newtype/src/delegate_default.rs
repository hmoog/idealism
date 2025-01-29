#[macro_export]
macro_rules! delegate_default {
    // Macro arm for newtypes with generics
    ($name:ident<$($generic:ident $(: $bound:tt)*),*>, $inner:ty) => {
        // Implement Default
        impl<$($generic $(: $bound)*),*> Default for $name<$($generic),*>
        where
            $inner: Default,
        {
            fn default() -> Self {
                Self(Default::default())
            }
        }
    };

    // Macro arm for newtypes without generics
    ($name:ident, $inner:ty) => {
        // Implement Default
        impl Default for $name
        where
            $inner: Default,
        {
            fn default() -> Self {
                Self(Default::default())
            }
        }
    };
}
