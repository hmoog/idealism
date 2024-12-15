use std::ops::Deref;
use std::sync::Arc;
use crate::rx::callback::CallbackOnce;

#[must_use]
pub struct ResourceGuard<T>(Arc<Inner<T>>);

pub struct Inner<T> {
    value: T,
    done_callback: Option<Box<dyn CallbackOnce<T>>>
}

impl<T> ResourceGuard<T> {
    pub fn new(value: T, done_callback: impl CallbackOnce<T>) -> Self {
        Self(Arc::new(Inner { value, done_callback: Some(Box::new(done_callback)) }))
    }

    pub fn get(&self) -> &T {
        &self.0.value
    }
}

impl <T> Clone for ResourceGuard<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> Deref for ResourceGuard<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0.value
    }
}

impl<T> Drop for ResourceGuard<T> {
    fn drop(&mut self) {
        if let Some(inner) = Arc::get_mut(&mut self.0) {
            if let Some(callback) = inner.done_callback.take() {
                callback(&inner.value);
            }
        }
    }
}