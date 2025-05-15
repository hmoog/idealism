use std::{ops::Deref, sync::Arc};

use common::{ids::IssuerID};
use config::{Config, ProtocolParams, ProtocolPlugins};
use protocol::{Protocol, ProtocolConfig};
use tracing::{Instrument, Span};
use validator::{Validator, ValidatorConfigParams};

pub struct Node {
    pub protocol: Arc<Protocol>,
    span: Span,
}

impl Node {
    pub fn new(span: Span, config_factory: impl Fn() -> Config) -> Self {
        Self {
            protocol: span.in_scope(|| Arc::new(Protocol::new(config_factory()))),
            span,
        }
    }

    pub fn new_validator(span: Span, issuer_id: IssuerID) -> Self {
        Self::new(span, move || {
            Config::default()
                .with_protocol_params(ProtocolParams::default().with_plugins(
                    ProtocolPlugins::Custom(|cfg, registry| {
                        ProtocolPlugins::Core.inject(cfg, registry);
                        registry.load::<Validator<Config>>();
                    }),
                ))
                .with_params(ValidatorConfigParams {
                    validator_id: issuer_id.clone(),
                })
        })
    }

    pub async fn run_for(self, duration: std::time::Duration) {
        self.start().instrument(self.span.clone()).await;
        tokio::time::sleep(duration).await;
        self.shutdown().await;
    }
}

impl Deref for Node {
    type Target = Arc<Protocol>;
    fn deref(&self) -> &Self::Target {
        &self.protocol
    }
}
