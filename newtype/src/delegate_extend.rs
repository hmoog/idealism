#[macro_export]
macro_rules! delegate_extend {
    // Macro arm for newtypes with generics
    ($name:ident<$($generic:ident $(: $bound:tt)*),*>, $inner:ty) => {
        // Implement Extend<I>
        impl<$($generic $(: $bound)*),*, I> Extend<I> for $name<$($generic),*>
        where
            $inner: Extend<I>,
        {
            fn extend<T: IntoIterator<Item = I>>(&mut self, iter: T) {
                self.0.extend(iter);
            }
        }
    };

    // Macro arm for newtypes without generics
    ($name:ident, $inner:ty) => {
        // Implement Extend<I>
        impl<I> Extend<I> for $name
        where
            $inner: Extend<I>,
        {
            fn extend<T: IntoIterator<Item = I>>(&mut self, iter: T) {
                self.0.extend(iter);
            }
        }
    };
}
