use std::sync::{Arc, atomic, atomic::AtomicU64};

use common::rx::Signal;

#[test]
fn test_signal() {
    let signal = Signal::new();

    let callback_counter = Arc::new(AtomicU64::new(0));

    let subscription = signal.subscribe({
        let callback_counter = callback_counter.clone();

        move |new_value: &i32| match &callback_counter.fetch_add(1, atomic::Ordering::SeqCst) {
            0 => assert_eq!(*new_value, 42),
            _ => panic!(),
        }
    });

    assert_eq!(*signal.get(), None);
    signal.set(42);
    assert_eq!(*signal.get(), Some(42));
    assert_eq!(callback_counter.load(atomic::Ordering::SeqCst), 1);

    signal.set(43);
    assert_eq!(*signal.get(), Some(42));
    assert_eq!(callback_counter.load(atomic::Ordering::SeqCst), 1);

    drop(subscription);

    signal.set(44);
    assert_eq!(*signal.get(), Some(42));
    assert_eq!(callback_counter.load(atomic::Ordering::SeqCst), 1);

    signal
        .subscribe({
            let callback_counter = callback_counter.clone();

            move |new_value: &i32| match callback_counter.fetch_add(1, atomic::Ordering::SeqCst) {
                1 => assert_eq!(*new_value, 42),
                _ => panic!(),
            }
        })
        .retain();
}
