#[macro_export]
macro_rules! delegate_from_iterator {
    // Macro arm for newtypes with generics
    ($name:ident<$($generic:ident $(: $bound:tt)*),*>, $inner:ty) => {
        // Implement FromIterator<I>
        impl<$($generic $(: $bound)*),*, I> FromIterator<I> for $name<$($generic),*>
        where
            $inner: FromIterator<I>,
        {
            fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
                Self(<$inner as FromIterator<I>>::from_iter(iter))
            }
        }
    };

    // Macro arm for newtypes without generics
    ($name:ident, $inner:ty) => {
        // Implement FromIterator<I>
        impl<I> FromIterator<I> for $name
        where
            $inner: FromIterator<I>,
        {
            fn from_iter<T: IntoIterator<Item = I>>(iter: T) -> Self {
                Self(<$inner as FromIterator<I>>::from_iter(iter))
            }
        }
    };
}
