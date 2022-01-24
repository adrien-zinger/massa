// Copyright (c) 2021 MASSA LABS <info@massa.net>

#![allow(clippy::assertions_on_constants)]

use massa_models::{Amount};
use massa_signature::PrivateKey;
use massa_time::MassaTime;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, usize};

/// Consensus full configuration (static + user defined)
///
/// Assert that `THREAD_COUNT >= 1 || T0.to_millis() >= 1 || T0.to_millis() % THREAD_COUNT == 0`
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LedgerConfig {
    /// Number of threads
    pub thread_count: u8,
    /// path to ledger db
    pub ledger_path: PathBuf,
    pub ledger_cache_capacity: u64,
    pub ledger_flush_interval: Option<MassaTime>,
}

impl From<&GraphConfig> for LedgerConfig {
    fn from(cfg: &GraphConfig) -> Self {
        LedgerConfig {
            thread_count: cfg.thread_count,
            ledger_path: cfg.ledger_path.clone(),
            ledger_cache_capacity: cfg.ledger_cache_capacity,
            ledger_flush_interval: cfg.ledger_flush_interval,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GraphConfig {
    /// Number of threads
    pub thread_count: u8,
    /// Private_key to sign genesis blocks.
    pub genesis_key: PrivateKey,
    /// Maximum number of blocks allowed in discarded blocks.
    pub max_discarded_blocks: usize,
    /// If a block  is future_block_processing_max_periods periods in the future, it is just discarded.
    pub future_block_processing_max_periods: u64,
    /// Maximum number of blocks allowed in FutureIncomingBlocks.
    pub max_future_processing_blocks: usize,
    /// Maximum number of blocks allowed in DependencyWaitingBlocks.
    pub max_dependency_blocks: usize,
    /// Threshold for fitness.
    pub delta_f0: u64,
    /// Maximum operation validity period count
    pub operation_validity_periods: u64,
    /// cycle duration in periods
    pub periods_per_cycle: u64,
    pub initial_ledger_path: PathBuf,
    pub block_reward: Amount,
    pub roll_price: Amount,
    /// force keep at least this number of final periods in RAM for each thread
    pub force_keep_final_periods: u64,
    pub endorsement_count: u32,
    //pub block_db_prune_interval: MassaTime,
    pub max_item_return_count: usize,

    // TODO: put this in an accessible config? It seems that all can be static
    /// path to ledger db (todo: static thing?)
    pub ledger_path: PathBuf,
    pub ledger_cache_capacity: u64,
    pub ledger_flush_interval: Option<MassaTime>,
}


/*
lazy_static::lazy_static! {
    /// Compact representation of key values of consensus algorithm used in API
    static ref STATIC_CONFIG: CompactConfig = CompactConfig {
        genesis_timestamp: *GENESIS_TIMESTAMP,
        end_timestamp: *END_TIMESTAMP,
        thread_count: THREAD_COUNT,
        t0: *T0,
        delta_f0: DELTA_F0,
        operation_validity_periods: OPERATION_VALIDITY_PERIODS,
        periods_per_cycle: PERIODS_PER_CYCLE,
        pos_lookback_cycles: POS_LOOKBACK_CYCLES,
        pos_lock_cycles: POS_LOCK_CYCLES,
        block_reward: *BLOCK_REWARD,
        roll_price: *ROLL_PRICE,
    };
}



impl GraphConfig {
    /// Utility method to derivate a compact configuration (for API use) from a full one
    pub fn compact_config(&self) -> CompactConfig {
        *STATIC_CONFIG
    }
}
*/