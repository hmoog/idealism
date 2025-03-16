use std::sync::{Arc, atomic, atomic::AtomicU64};

use types::rx::Variable;

#[test]
fn test_variable() {
    let variable = Variable::new();

    let callback_counter = Arc::new(AtomicU64::new(0));

    let subscription = variable.subscribe({
        let callback_counter = callback_counter.clone();

        move |update: &(Option<i32>, Option<i32>)| {
            let (old_value, new_value) = update;

            match callback_counter.fetch_add(1, atomic::Ordering::SeqCst) {
                0 => {
                    assert_eq!(*old_value, None);
                    assert_eq!(*new_value, Some(42));
                }
                1 => {
                    assert_eq!(*old_value, Some(42));
                    assert_eq!(*new_value, Some(43));
                }
                _ => panic!(),
            }
        }
    });

    variable.set(42);
    assert_eq!(variable.get().unwrap(), 42);
    assert_eq!(callback_counter.load(atomic::Ordering::SeqCst), 1);

    variable.set(43);
    assert_eq!(variable.get().unwrap(), 43);
    assert_eq!(callback_counter.load(atomic::Ordering::SeqCst), 2);

    drop(subscription);

    variable.set(44);
    let val = variable.get().unwrap();
    assert_eq!(val, 44);
    assert_eq!(variable.get().unwrap(), 44);
    assert_eq!(callback_counter.load(atomic::Ordering::SeqCst), 2);
}
