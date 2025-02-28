use crate::hasher::Hasher;

pub trait Hashable {
    fn hash<H: Hasher>(&self, hasher: &mut H);
}
