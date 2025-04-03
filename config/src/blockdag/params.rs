use common::ids::BlockID;

#[derive(Default)]
pub struct BlockDAGParams {
    pub(crate) genesis_block_id: BlockID,
}
