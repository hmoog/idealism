use crate::hash::Hasher;

pub trait Hashable {
    fn hash<H: Hasher>(&self, hasher: &mut H);
}
