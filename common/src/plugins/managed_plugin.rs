use std::sync::Arc;
use crate::plugins::Plugin;
use crate::plugins::plugins::Plugins;

pub trait ManagedPlugin<Trait: ?Sized>: Sized {
    fn construct(plugins: &mut Plugins<Trait>) -> Arc<Self>;

    fn shutdown(&self);

    fn downcast(arc: Arc<Self>) -> Arc<Trait>;
}

impl<T: ManagedPlugin<Trait>, Trait: ?Sized> Plugin<Trait> for T {
    fn shutdown(&self) {
        ManagedPlugin::shutdown(self);
    }

    fn downcast(arc: Arc<Self>) -> Arc<Trait> {
        ManagedPlugin::downcast(arc)
    }
}
