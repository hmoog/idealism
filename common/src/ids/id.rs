use std::{
    fmt::{Debug, Display},
    hash,
    hash::Hash,
    marker::PhantomData,
    ops::Deref,
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use crate::hash::{Hashable, Hasher};

#[derive(Deserialize, Serialize)]
pub struct Id<H: Hasher>(Arc<[u8; 32]>, PhantomData<H>);

impl<H: Hasher> Id<H> {
    pub fn new<T: Hashable>(value: &T) -> Self {
        let mut hasher = H::new();
        value.hash(&mut hasher);
        Id(Arc::new(hasher.finalize()), PhantomData)
    }
}

impl<H: Hasher, T: Hashable> From<T> for Id<H> {
    fn from(value: T) -> Self {
        Id::new(&value)
    }
}

impl<H: Hasher> From<[u8; 32]> for Id<H> {
    fn from(value: [u8; 32]) -> Self {
        Id(Arc::new(value), PhantomData)
    }
}

impl<T: Hasher> Default for Id<T> {
    fn default() -> Self {
        Id(Arc::new([0; 32]), PhantomData)
    }
}

impl<T: Hasher> Display for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bytes = &self.0;
        if bytes.len() < 6 {
            // fallback in case of very short IDs
            write!(f, "0x")?;
            for b in bytes.iter() {
                write!(f, "{:02x}", b)?;
            }
        } else {
            write!(
                f,
                "0x{:02x}{:02x}{:02x}..{:02x}{:02x}{:02x}",
                bytes[0],
                bytes[1],
                bytes[2],
                bytes[bytes.len() - 3],
                bytes[bytes.len() - 2],
                bytes[bytes.len() - 1]
            )?;
        }
        Ok(())
    }
}

impl<T: Hasher> Debug for Id<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "0x")?;
        for byte in self.0.iter() {
            write!(f, "{:02x}", byte)?;
        }
        Ok(())
    }
}

impl<H: Hasher> Clone for Id<H> {
    fn clone(&self) -> Self {
        Id(Arc::clone(&self.0), PhantomData)
    }
}

impl<H: Hasher> Deref for Id<H> {
    type Target = [u8; 32];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<H: Hasher> PartialEq for Id<H> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || *self.0 == *other.0
    }
}

impl<H: Hasher> Eq for Id<H> {}

impl<H: Hasher> Hash for Id<H> {
    fn hash<T: hash::Hasher>(&self, state: &mut T) {
        self.0.hash(state);
    }
}
