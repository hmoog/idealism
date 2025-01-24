use std::sync::{Arc, Mutex};

use slotmap::HopSlotMap;

use crate::rx::{
    callback::{Callback, Callbacks},
    subscription::{ID, Subscription},
};

#[derive(Clone)]
pub struct Event<T>(Arc<Callbacks<T>>);

impl<T> Event<T> {
    pub fn new() -> Self {
        Self(Arc::new(Mutex::new(HopSlotMap::with_key())))
    }

    pub fn subscribe(&self, callback: impl Callback<T>) -> Subscription<Callbacks<T>> {
        Subscription::new(Arc::downgrade(&self.0), Some(self.add_callback(callback)))
    }

    pub fn trigger(&self, event: &T) {
        for (_, callback) in self.0.lock().unwrap().iter_mut() {
            callback(event);
        }
    }

    fn add_callback(&self, callback: impl Callback<T>) -> ID {
        self.0.lock().unwrap().insert(Box::new(callback))
    }
}

impl<T> Default for Event<T> {
    fn default() -> Self {
        Self::new()
    }
}
