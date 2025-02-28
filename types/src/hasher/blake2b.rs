use blake2::{Blake2b, Digest};

pub struct Hasher(Blake2b<blake2::digest::consts::U32>);

impl super::Hasher for Hasher {
    fn new() -> Self {
        Hasher(Blake2b::<blake2::digest::consts::U32>::new())
    }

    fn update(&mut self, data: &[u8]) {
        self.0.update(data);
    }

    fn finalize(self) -> [u8; 32] {
        let hash = self.0.finalize();
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(&hash[..32]);
        hash_bytes
    }
}
