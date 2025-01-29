#[macro_export]
macro_rules! delegate {
    ($name:ident<$($generic:ident $(: $bound:tt)*),*>, $inner:ty, $($Trait:ident),+) => {
        $(
            $crate::delegate_one_trait!($Trait, $name<$($generic),*>, $inner);
        )+
    };

    // Case without generics
    ($name:ident, $inner:ty, $($Trait:ident),+) => {
        $(
            $crate::delegate_one_trait!($Trait, $name, $inner);
        )+
    };
}

#[macro_export]
macro_rules! delegate_one_trait {
    (Default, $Type:ty, $Collection:ty) => {
        $crate::delegate_default!($Type, $Collection);
    };
    (FromIterator, $Type:ty, $Collection:ty) => {
        $crate::delegate_from_iterator!($Type, $Collection);
    };
    (IntoIterator, $Type:ty, $Collection:ty) => {
        $crate::delegate_into_iterator!($Type, $Collection);
    };
}
