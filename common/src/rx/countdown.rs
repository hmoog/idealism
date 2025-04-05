use std::{
    ops::Deref,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::rx::Signal;

pub struct Countdown {
    signal: Signal<()>,
    counter: AtomicUsize,
}

impl Countdown {
    pub fn new(counter: usize) -> Self {
        Countdown {
            signal: {
                let signal = Signal::default();
                if counter == 0 {
                    signal.set(());
                }
                signal
            },
            counter: AtomicUsize::new(counter),
        }
    }

    pub fn decrease(&self) {
        if self.counter.fetch_sub(1, Ordering::SeqCst) - 1 == 0 {
            self.signal.set(());
        }
    }
}

impl Deref for Countdown {
    type Target = Signal<()>;

    fn deref(&self) -> &Self::Target {
        &self.signal
    }
}
