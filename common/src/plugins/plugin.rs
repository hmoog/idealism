use std::sync::Arc;

pub trait Plugin<Trait: ?Sized>: Sized {
    fn shutdown(&self);

    fn downcast(arc: Arc<Self>) -> Arc<Trait>;
}
