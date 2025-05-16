use common::ids::IssuerID;
use networking::Networking;
use sim::{Network, Node};
use tracing::info_span;
use tracing_subscriber::{EnvFilter, fmt};

#[tokio::test]
async fn test_protocol() {
    let _ = fmt()
        .with_target(false)
        .with_env_filter(EnvFilter::new("trace"))
        .with_test_writer()
        .try_init();

    let nodes = [
        Node::new_validator(info_span!("node1"), IssuerID::from([1; 32])),
        Node::new_validator(info_span!("node2"), IssuerID::from([2; 32])),
        Node::new_validator(info_span!("node3"), IssuerID::from([3; 32])),
        Node::new_validator(info_span!("node4"), IssuerID::from([4; 32])),
    ];

    let network = Network::default();

    let mut node_handles = Vec::new();
    for node in nodes {
        let _ = node
            .plugins
            .get::<Networking>()
            .unwrap()
            .connect(&network)
            .await;

        node_handles.push(tokio::spawn(
            node.run_for(std::time::Duration::from_secs(1)),
        ));
    }

    for node in node_handles {
        let _ = node.await;
    }
}
