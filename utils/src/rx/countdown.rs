use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

pub struct Countdown {
    inner: Arc<Inner>,
}

struct Inner {
    count: AtomicUsize,
    target: usize,
    callback: Box<dyn Fn() + Send + Sync>,
}

impl Countdown {
    pub fn new(x: usize, callback: impl Fn() + Send + Sync + 'static) -> Self {
        Countdown {
            inner: Arc::new(Inner {
                count: AtomicUsize::new(0),
                target: x,
                callback: match x {
                    0 => {
                        callback();
                        Box::new(|| {})
                    }
                    _ => Box::new(callback),
                },
            }),
        }
    }

    pub fn decrease(&self) {
        if self.inner.count.fetch_add(1, Ordering::SeqCst) + 1 == self.inner.target {
            (self.inner.callback)();
        }
    }
}

impl Clone for Countdown {
    fn clone(&self) -> Self {
        Countdown {
            inner: Arc::clone(&self.inner),
        }
    }
}
