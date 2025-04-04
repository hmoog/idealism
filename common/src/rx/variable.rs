use std::sync::{Mutex, MutexGuard};

use crate::rx::{
    Event,
    UpdateType::{Notify, Retain},
    callback::{Callback, Callbacks},
    subscription::Subscription,
};

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
        let _ = self.compute::<(), _>(|value| Notify(value, Some(new_value)));
    }

    pub fn set_if_none_or<F: FnOnce(&T, &T) -> bool>(&self, new: T, cond: F) {
        let _ = self.compute::<(), _>(move |current| match current {
            Some(old) if !cond(&old, &new) => Retain(Some(old)),
            _ => Notify(current, Some(new)),
        });
    }

    pub fn unset(&self) {
        let _ = self.compute::<(), _>(|value| {
            if value.is_none() {
                Retain(value)
            } else {
                Notify(value, None)
            }
        });
    }

    pub fn get(&self) -> MutexGuard<Option<T>> {
        self.value.lock().unwrap()
    }

    pub fn read(&self, f: impl FnOnce(Option<&T>)) {
        f(self.value.lock().unwrap().as_ref())
    }

    pub fn must_read(&self, f: impl FnOnce(&T)) {
        f(self.value.lock().unwrap().as_ref().unwrap())
    }

    pub fn get_or_insert(&self, default: T) -> MutexGuard<Option<T>> {
        self.compute_if_none(|| self.process_update(None, Some(default)))
    }

    pub fn get_or_insert_with(&self, default: impl FnOnce() -> T) -> MutexGuard<Option<T>> {
        self.compute_if_none(|| self.process_update(None, Some(default())))
    }

    pub fn subscribe(
        &self,
        mut callback: impl Callback<(Option<T>, Option<T>)>,
    ) -> Subscription<Callbacks<(Option<T>, Option<T>)>> {
        let _lock = self.compute_if_some(|current_value| {
            let update = (None, Some(current_value));
            callback(&update);
            update.1
        });

        self.event.subscribe(callback)
    }

    pub fn attach(&self, callback: impl Callback<(Option<T>, Option<T>)>) {
        self.subscribe(callback).retain()
    }

    pub fn compute<E, F: FnOnce(Option<T>) -> UpdateType<T, E>>(
        &self,
        compute: F,
    ) -> Result<(), E> {
        let mut error = None;
        let mut locked_value = self.get();

        *locked_value = match compute(locked_value.take()) {
            Retain(value) => value,
            Notify(old, new) => self.process_update(old, new),
            UpdateType::Error(old, err) => {
                error = Some(err);
                old
            }
        };

        if let Some(err) = error {
            return Err(err);
        }

        Ok(())
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

impl<T: Ord> Variable<T> {
    pub fn track_max(&self, new: T) {
        let _ = self.compute::<(), _>(move |current| match current {
            Some(old) if old >= new => Retain(Some(old)),
            _ => Notify(current, Some(new)),
        });
    }
}

impl<T> Default for Variable<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub enum UpdateType<T, E> {
    Retain(Option<T>),
    Notify(Option<T>, Option<T>),
    Error(Option<T>, E),
}
