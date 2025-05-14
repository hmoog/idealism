use std::sync::Arc;

use common::{extensions::ArcExt, ids::IssuerID, up};
use config::{Config, ProtocolParams, ProtocolPlugins};
use protocol::{Protocol, ProtocolConfig};
use tracing::{Instrument, Span, info_span};
use tracing_subscriber::{EnvFilter, fmt};
use networking::Networking;
use sim::Network;
use validator::{Validator, ValidatorConfigParams};

pub struct TestNode {
    pub protocol: Arc<Protocol>,
    span: Span,
}

impl TestNode {
    pub fn new(span: Span, config_factory: impl Fn() -> Config) -> Self {
        Self {
            protocol: span.in_scope(|| Arc::new(Protocol::new(config_factory()))),
            span,
        }
    }

    pub fn new_validator(span: Span, issuer_id: IssuerID) -> Self {
        Self::new(
            span,
            move || Config::default()
                .with_protocol_params(ProtocolParams::default().with_plugins(
                    ProtocolPlugins::Custom(|cfg, registry| {
                        ProtocolPlugins::Core.inject(cfg, registry);
                        registry.load::<Validator<Config>>();
                    }),
                ))
                .with_params(ValidatorConfigParams {
                    validator_id: issuer_id.clone(),
                }),
        )
    }

    pub async fn run_for(self, duration: std::time::Duration) {
        let protocol = self.protocol.downgrade();
        tokio::spawn(
            async move {
                up!(protocol: {
                    tokio::time::sleep(duration).await;
                    protocol.shutdown();
                });
            }
            .instrument(self.span.clone()),
        );
        self.protocol.start().instrument(self.span.clone()).await;
    }
}

#[tokio::test]
async fn test_protocol() {
    let _ = fmt()
        .with_env_filter(EnvFilter::new("info"))
        .with_test_writer()
        .try_init();

    let nodes = [
        TestNode::new_validator(info_span!("node1"), IssuerID::from([1; 32])),
        TestNode::new_validator(info_span!("node2"), IssuerID::from([2; 32])),
        TestNode::new_validator(info_span!("node3"), IssuerID::from([3; 32])),
        TestNode::new_validator(info_span!("node4"), IssuerID::from([4; 32])),
    ];

    let network = Network::default();

    let mut run_handles = Vec::new();
    for node in nodes {
        let _ = node.protocol.plugins.get::<Networking>().unwrap().connect(&network).await;

        run_handles.push(tokio::spawn(node.run_for(std::time::Duration::from_secs(1))));
    }

    for handle in run_handles {
        let _ = handle.await;
    }
}
