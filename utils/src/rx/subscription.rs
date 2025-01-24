use std::sync::Weak;

use slotmap::new_key_type;

new_key_type! {
    pub struct ID;
}

#[must_use = "If unused, the subscription will be cancelled immediately. Use .forever() to keep it alive until the target is dropped or store the Subscription in a variable."]
pub struct Subscription<T: Unsubscribable> {
    pub(crate) callbacks: Weak<T>,
    pub(crate) id: Option<ID>,
}

impl<T: Unsubscribable> Subscription<T> {
    pub fn new(callbacks: Weak<T>, id: Option<ID>) -> Self {
        Self { callbacks, id }
    }

    pub fn forever(mut self) {
        self.id = None;
    }
}

pub trait Unsubscribable {
    fn unsubscribe(&self, key: ID);
}

impl<T: Unsubscribable> Drop for Subscription<T> {
    fn drop(&mut self) {
        if let Some(id) = self.id.take() {
            if let Some(emitter) = self.callbacks.upgrade() {
                emitter.unsubscribe(id);
            }
        }
    }
}
