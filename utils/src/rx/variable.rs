use std::sync::{Mutex, MutexGuard};
use crate::rx::callback::{Callback, Callbacks};
use crate::rx::Event;
use crate::rx::subscription::Subscription;

pub struct Variable<T> {
    value: Mutex<Option<T>>,
    event: Event<(Option<T>, Option<T>)>,
}

impl<T> Variable<T> {
    pub fn new() -> Self {
        Self {
            value: Mutex::new(None),
            event: Event::new(),
        }
    }

    pub fn set(&self, new_value: T) {
        drop(self.compute(|value| self.process_update(value, Some(new_value))));
    }

    pub fn unset(&self) {
        drop(self.compute(|value| self.process_update(value, None)));
    }

    pub fn get(&self) -> MutexGuard<Option<T>> {
        self.value.lock().unwrap()
    }

    pub fn get_or_insert(&self, default: T) -> MutexGuard<Option<T>> {
        self.compute_if_none(|| self.process_update(None, Some(default)))
    }

    pub fn get_or_insert_with(&self, default: impl FnOnce() -> T) -> MutexGuard<Option<T>> {
        self.compute_if_none(|| self.process_update(None, Some(default())))
    }

    pub fn subscribe(&self, mut callback: impl Callback<(Option<T>, Option<T>)>) -> Subscription<Callbacks<(Option<T>, Option<T>)>> {
        let _lock = self.compute_if_some(|current_value| {
            let update = (None, Some(current_value));
            callback(&update);
            update.1
        });

        self.event.subscribe(callback)
    }

    fn compute<F: FnOnce(Option<T>) -> Option<T>>(&self, compute: F) -> MutexGuard<Option<T>> {
        let mut value = self.get();
        *value = compute(if value.is_none() { None } else { Some(value.take().unwrap()) });
        value
    }

    fn compute_if_some<F: FnOnce(T) -> Option<T>>(&self, compute: F) -> MutexGuard<Option<T>> {
        let mut value = self.get();
        if value.is_some() {
            *value = compute(value.take().unwrap());
        }
        value
    }

    fn compute_if_none<F: FnOnce() -> Option<T>>(&self, compute: F) -> MutexGuard<Option<T>> {
        let mut value = self.get();
        if value.is_none() {
            *value = compute();
        }
        value
    }

    fn process_update(&self, old_value: Option<T>, new_value: Option<T>) -> Option<T> {
        let update = (old_value, new_value);
        self.event.trigger(&update);
        update.1
    }
}

impl<T> Default for Variable<T> {
    fn default() -> Self {
        Self::new()
    }
}