pub mod bft {
    mod committee;
    mod member;
    mod members;

    pub use committee::Committee;
    pub use member::Member;
    pub use members::Members;
}
pub mod blocks {
    mod block;
    mod block_metadata;
    mod block_metadata_ref;
    mod network_block;

    pub use block::Block;
    pub use block_metadata::BlockMetadata;
    pub use block_metadata_ref::BlockMetadataRef;
    pub use network_block::NetworkBlock;
}
pub mod collections {
    mod any_map;
    mod max_set;

    pub use any_map::AnyMap;
    pub use max_set::MaxSet;
}
pub mod extensions {
    mod arc;

    pub use arc::*;
}
pub mod errors {
    mod error;
    mod result;

    pub use error::*;
    pub use result::*;
}
pub mod hash {
    mod blake2b;
    mod hashable;
    mod hasher;

    pub use blake2b::Hasher as Blake2bHasher;
    pub use hashable::Hashable;
    pub use hasher::Hasher;
}
pub mod ids {
    mod block_id;
    mod id;
    mod issuer_id;

    pub use block_id::BlockID;
    pub use id::Id;
    pub use issuer_id::IssuerID;
}
pub mod rx {
    mod callback;
    mod countdown;
    mod event;
    mod resource_guard;
    mod signal;
    mod subscription;
    mod variable;

    pub use callback::*;
    pub use countdown::*;
    pub use event::*;
    pub use resource_guard::*;
    pub use signal::*;
    pub use subscription::*;
    pub use variable::*;
}

#[macro_export]
macro_rules! up {
    ( $( $var:ident ),+ : $body:expr ) => {{
        $( let Some($var) = $var.upgrade() else { return; }; )+
        $body
    }};
}

#[macro_export]
macro_rules! with {
    // Match mixed vars: `a`, `(mut b)`, `c`, etc.
    ( $( $var:tt ),+ : $body:expr ) => {{
        $(
            with!(@bind $var);
        )+
        $body
    }};

    // Internal rule to bind immutable: plain ident
    (@bind $var:ident) => {
        let $var = $var.clone();
    };

    // Internal rule to bind mutable: (mut ident)
    (@bind (mut $var:ident)) => {
        let mut $var = $var.clone();
    };
}

#[macro_export]
macro_rules! down {
    ( $( $var:ident ),+ : $body:expr ) => {{
        $( let $var = $var.downgrade(); )+
        $body
    }};
}
