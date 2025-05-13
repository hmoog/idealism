use std::sync::Arc;

use common::{extensions::ArcExt, ids::IssuerID, up};
use config::{Config, ProtocolParams, ProtocolPlugins};
use protocol::{Protocol, ProtocolConfig};
use tracing::{Instrument, Level, Span, span};
use tracing_subscriber::{EnvFilter, fmt};
use networking::Networking;
use sim::Network;
use validator::{Validator, ValidatorConfigParams};

pub struct TestNode {
    pub protocol: Arc<Protocol>,
    span: Span,
}

impl TestNode {
    pub fn new(name: &str, config: Config) -> Self {
        let span = span!(Level::INFO, "node", name = %name);
        Self {
            protocol: span.in_scope(|| Arc::new(Protocol::new(config))),
            span,
        }
    }

    pub fn new_default_validator(name: &str, issuer_id: IssuerID) -> Self {
        Self::new(
            name,
            Config::default()
                .with_protocol_params(ProtocolParams::default().with_plugins(
                    ProtocolPlugins::Custom(|cfg, registry| {
                        ProtocolPlugins::Core.inject(cfg, registry);
                        registry.load::<Validator<Config>>();
                    }),
                ))
                .with_params(ValidatorConfigParams {
                    validator_id: issuer_id,
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
        .with_env_filter(EnvFilter::new("trace"))
        .with_test_writer()
        .try_init();

    let network = Network::default();

    let mut run_handles = Vec::new();
    for i in 1..5 {
        println!("Starting node {}", i);
        let test_node =
            TestNode::new_default_validator(&format!("node{}", i), IssuerID::from([i as u8; 32]));

        let _ = test_node.protocol.plugins.get::<Networking>().unwrap().connect(&network).await;

        let handle = tokio::spawn(test_node.run_for(std::time::Duration::from_secs(1)));
        run_handles.push(handle);
    }

    for handle in run_handles {
        let _ = handle.await;
    }
}
