use std::sync::Arc;

use common::{blocks::BlockMetadata, rx::Signal};

pub type Address = Arc<Signal<BlockMetadata>>;
