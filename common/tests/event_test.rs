use common::rx::Event;

#[test]
fn test_add() {
    let event = Event::new();

    let _subscription = event.subscribe(|t: &String| {
        println!("CLOSURE1: {:}", t);
    });

    {
        let _subscription = event.subscribe(|t: &String| {
            println!("CLOSURE2: {:}", t);
        });

        event.trigger(&"Hello1".to_string());
        event.trigger(&"Hello2".to_string());
    }

    event.trigger(&"Hello3".to_string());
}
