#[macro_export]
macro_rules! delegate_into_iterator {
    // Macro arm for newtypes with generics
    ($name:ident<$($generic:ident $(: $bound:tt)*),*>, $inner:ty) => {
        // Implement IntoIterator (owned)
        impl<$($generic $(: $bound)*),*> IntoIterator for $name<$($generic),*>
        where
            $inner: IntoIterator,
        {
            type Item = <$inner as IntoIterator>::Item;
            type IntoIter = <$inner as IntoIterator>::IntoIter;

            fn into_iter(self) -> Self::IntoIter {
                self.0.into_iter()
            }
        }

        // Implement IntoIterator for &T (immutable borrow) with explicit lifetime
        impl<'a, $($generic $(: $bound)*),*> IntoIterator for &'a $name<$($generic),*>
        where
            &'a $inner: IntoIterator,
        {
            type Item = <&'a $inner as IntoIterator>::Item;
            type IntoIter = <&'a $inner as IntoIterator>::IntoIter;

            fn into_iter(self) -> Self::IntoIter {
                (&self.0).into_iter()
            }
        }

        // Implement IntoIterator for &mut T (mutable borrow) with explicit lifetime
        impl<'a, $($generic $(: $bound)*),*> IntoIterator for &'a mut $name<$($generic),*>
        where
            &'a mut $inner: IntoIterator,
        {
            type Item = <&'a mut $inner as IntoIterator>::Item;
            type IntoIter = <&'a mut $inner as IntoIterator>::IntoIter;

            fn into_iter(self) -> Self::IntoIter {
                (&mut self.0).into_iter()
            }
        }
    };

    // Macro arm for newtypes without generics
    ($name:ident, $inner:ty) => {
        // Implement IntoIterator (owned)
        impl IntoIterator for $name
        where
            $inner: IntoIterator,
        {
            type Item = <$inner as IntoIterator>::Item;
            type IntoIter = <$inner as IntoIterator>::IntoIter;

            fn into_iter(self) -> Self::IntoIter {
                self.0.into_iter()
            }
        }

        // Implement IntoIterator for &T (immutable borrow) with explicit lifetime
        impl<'a> IntoIterator for &'a $name
        where
            &'a $inner: IntoIterator,
        {
            type Item = <&'a $inner as IntoIterator>::Item;
            type IntoIter = <&'a $inner as IntoIterator>::IntoIter;

            fn into_iter(self) -> Self::IntoIter {
                (&self.0).into_iter()
            }
        }

        // Implement IntoIterator for &mut T (mutable borrow) with explicit lifetime
        impl<'a> IntoIterator for &'a mut $name
        where
            &'a mut $inner: IntoIterator,
        {
            type Item = <&'a mut $inner as IntoIterator>::Item;
            type IntoIter = <&'a mut $inner as IntoIterator>::IntoIter;

            fn into_iter(self) -> Self::IntoIter {
                (&mut self.0).into_iter()
            }
        }
    };
}
