use std::path::PathBuf;

use massa_models::Amount;
use massa_signature::PrivateKey;
use num::rational::Ratio;
use serde::{Deserialize, Serialize};



#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProofOfStakeConfig {
    ///// Time in millis when the blockclique started.
    //pub genesis_timestamp: MassaTime,
    ///// TESTNET: time when the blockclique is ended.
    //pub end_timestamp: Option<MassaTime>,
    /// Number of threads
    pub thread_count: u8,
    ///// Time between the periods in the same thread.
    //pub t0: MassaTime,
    /// Private_key to sign genesis blocks.
    pub genesis_key: PrivateKey,
    ///// Staking private keys
    //pub staking_keys_path: PathBuf,
    ///// Maximum number of blocks allowed in discarded blocks.
    //pub max_discarded_blocks: usize,
    ///// If a block  is future_block_processing_max_periods periods in the future, it is just discarded.
    //pub future_block_processing_max_periods: u64,
    ///// Maximum number of blocks allowed in FutureIncomingBlocks.
    //pub max_future_processing_blocks: usize,
    ///// Maximum number of blocks allowed in DependencyWaitingBlocks.
    //pub max_dependency_blocks: usize,
    ///// Threshold for fitness.
    //pub delta_f0: u64,
    ///// Maximum number of operations per block
    //pub max_operations_per_block: u32,
    ///// Maximum tries to fill a block with operations
    //pub max_operations_fill_attempts: u32,
    ///// Maximum block size in bytes
    //pub max_block_size: u32,
    ///// Maximum operation validity period count
    //pub operation_validity_periods: u64,
    /// cycle duration in periods
    pub periods_per_cycle: u64,
    /// PoS lookback cycles: when drawing for cycle N, we use the rolls from cycle N - pos_lookback_cycles - 1
    pub pos_lookback_cycles: u64,
    /// PoS lock cycles: when some rolls are released, we only credit the coins back to their owner after waiting  pos_lock_cycles
    pub pos_lock_cycles: u64,
    /// number of cached draw cycles for PoS
    pub pos_draw_cached_cycles: usize,
    /// number of cycle misses (strictly) above which stakers are deactivated
    pub pos_miss_rate_deactivation_threshold: Ratio<u64>,
    ///// path to ledger db
    //pub ledger_path: PathBuf,
    //pub ledger_cache_capacity: u64,
    //pub ledger_flush_interval: Option<MassaTime>,
    //pub ledger_reset_at_startup: bool,
    //pub initial_ledger_path: PathBuf,
    //pub block_reward: Amount,
    //pub operation_batch_size: usize,
    pub initial_rolls_path: PathBuf,
    pub initial_draw_seed: String,
    pub roll_price: Amount,
    ///// stats timespan
    //pub stats_timespan: MassaTime,
    ///// max event send wait
    //pub max_send_wait: MassaTime,
    ///// force keep at least this number of final periods in RAM for each thread
    //pub force_keep_final_periods: u64,
    pub endorsement_count: u32,
    //pub block_db_prune_interval: MassaTime,
    //pub max_item_return_count: usize,
    ///// If we want to generate blocks.
    ///// Parameter that shouldn't be defined in prod.
    //#[serde(skip, default = "Default::default")]
    //pub disable_block_creation: bool,
}