use crate::ProtocolPlugins;

#[derive(Default)]
pub struct ProtocolParams {
    pub(crate) plugins: ProtocolPlugins,
}
