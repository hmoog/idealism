use std::sync::Arc;

use crate::plugins::manager::Manager;

pub trait Plugin<Trait: ?Sized>: Sized {
    fn construct(manager: &mut Manager<Trait>) -> Self;

    fn plugin(arc: Arc<Self>) -> Arc<Trait>
    where
        Self: Sized;
}
