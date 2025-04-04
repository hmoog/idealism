use crate::ProtocolPlugins;

#[derive(Default)]
pub struct ProtocolParams {
    pub(crate) plugins: ProtocolPlugins,
}

impl ProtocolParams {
    pub fn with_plugins(mut self, plugins: ProtocolPlugins) -> Self {
        self.plugins = plugins;
        self
    }
}
