use std::sync::{Arc, Mutex, MutexGuard};

use slotmap::HopSlotMap;

use crate::rx::{
    callback::{CallbackOnce, CallbacksOnce},
    subscription::{ID, Subscription},
};

pub struct Signal<T> {
    signal: Mutex<Option<T>>,
    callbacks: Arc<CallbacksOnce<T>>,
}

impl<T> Signal<T> {
    pub fn set(&self, signal: T) {
        drop(self.get_or_insert(signal));
    }

    pub fn get(&self) -> MutexGuard<Option<T>> {
        self.signal.lock().unwrap()
    }

    pub fn get_or_insert(&self, default: T) -> MutexGuard<Option<T>> {
        self.get_or_insert_with(|| default)
    }

    pub fn get_or_insert_with(&self, default: impl FnOnce() -> T) -> MutexGuard<Option<T>> {
        let mut value = self.signal.lock().unwrap();
        if value.is_none() {
            let signal = default();
            for (_, callback) in self.callbacks.lock().unwrap().drain() {
                callback(&signal);
            }
            *value = Some(signal);
        }
        value
    }

    pub fn subscribe(&self, callback: impl CallbackOnce<T>) -> Subscription<CallbacksOnce<T>> {
        Subscription::new(
            Arc::downgrade(&self.callbacks),
            self.try_add_callback(callback),
        )
    }

    pub fn attach(&self, callback: impl CallbackOnce<T>) {
        Subscription::new(
            Arc::downgrade(&self.callbacks),
            self.try_add_callback(callback),
        )
        .retain()
    }

    fn try_add_callback(&self, callback: impl CallbackOnce<T>) -> Option<ID> {
        match self.signal.lock().unwrap().as_ref() {
            Some(emitted_signal) => {
                callback(emitted_signal);
                None
            }
            None => Some(self.callbacks.lock().unwrap().insert(Box::new(callback))),
        }
    }
}

impl<T: Clone> Signal<T> {
    pub fn value(&self) -> Option<T> {
        self.get().as_ref().cloned()
    }
}

impl<T> Default for Signal<T> {
    fn default() -> Self {
        Self {
            signal: Mutex::new(None),
            callbacks: Arc::new(Mutex::new(HopSlotMap::with_key())),
        }
    }
}
