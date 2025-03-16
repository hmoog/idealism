use crate::{
    hash::{Hashable, Hasher},
    ids::{BlockID, IssuerID},
};

pub struct NetworkBlock {
    pub parents: Vec<BlockID>,
    pub issuer_id: IssuerID,
}

impl Hashable for NetworkBlock {
    fn hash<H: Hasher>(&self, hasher: &mut H) {
        hasher.update(&self.parents.len().to_be_bytes());
        for parent in &self.parents {
            hasher.update(parent.as_slice());
        }
    }
}
