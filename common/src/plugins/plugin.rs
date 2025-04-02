use std::sync::Arc;

use crate::plugins::plugin_registry::PluginRegistry;

pub trait Plugin<Trait: ?Sized>: Sized {
    fn construct(manager: &mut PluginRegistry<Trait>) -> Arc<Self>;

    fn plugin(arc: Arc<Self>) -> Arc<Trait>;
}
