use std::sync::Mutex;
use slotmap::HopSlotMap;
use trait_set::trait_set;
use crate::rx::subscription::{Unsubscribable, ID};

trait_set! {
    pub trait Callback<T> = FnMut(&T) + Send + Sync + 'static;

    pub trait CallbackOnce<T> = FnOnce(&T) + Send + Sync + 'static;
}

pub type Callbacks<T> = Mutex<HopSlotMap<ID, Box<dyn Callback<T>>>>;

impl<T> Unsubscribable for Callbacks<T> {
    fn unsubscribe(&self, key: ID) {
        self.lock().unwrap().remove(key);
    }
}

pub type CallbacksOnce<T> = Mutex<HopSlotMap<ID, Box<dyn CallbackOnce<T>>>>;

impl<T> Unsubscribable for CallbacksOnce<T> {
    fn unsubscribe(&self, key: ID) {
        self.lock().unwrap().remove(key);
    }
}