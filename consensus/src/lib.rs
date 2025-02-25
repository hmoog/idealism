mod block;
mod error;
mod issuer_id;
mod protocol;
mod protocol_data;

#[cfg(test)]
mod tests {
    use virtual_voting::builtin::DefaultConfig;

    use crate::protocol::Protocol;

    #[test]
    fn test_message() {
        let mut protocol = Protocol::new(DefaultConfig::new());

        protocol.run();
    }
}
