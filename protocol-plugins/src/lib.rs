pub mod consensus {
    mod accepted_blocks;
    mod plugin;

    pub use accepted_blocks::AcceptedBlocks;
    pub use plugin::Consensus;
}

pub mod consensus_round {
    mod plugin;

    pub use plugin::ConsensusRound;
}
