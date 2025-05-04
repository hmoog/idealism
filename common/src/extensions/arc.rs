use std::sync::{Arc, Weak};

pub trait ArcExt {
    type Target;

    fn downgrade(&self) -> Weak<Self::Target>;
}

impl<T> ArcExt for Arc<T> {
    type Target = T;

    fn downgrade(&self) -> Weak<T> {
        Arc::downgrade(self)
    }
}
