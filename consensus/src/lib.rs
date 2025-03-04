mod error;
mod events;
mod issuer_id;
mod protocol;
mod protocol_data;
mod types;

#[cfg(test)]
mod tests {
    use virtual_voting::builtin::DefaultConfig;

    use crate::protocol::Protocol;

    #[test]
    fn test_message() {
        let mut protocol = Protocol::new(DefaultConfig::new());

        protocol
            .blocks_ordered
            .subscribe(|event| {
                println!("Blocks ordered: {:?}", event);
            })
            .forever();

        protocol.run();
    }
}
