use std::{path::Path, str::FromStr, collections::HashMap};

use massa_consensus::{ConsensusConfig, RollCounts, RollUpdate, RollUpdates};
use massa_models::{Amount, Address, ledger::LedgerData};
use massa_signature::{generate_random_private_key, PrivateKey, derive_public_key};
use massa_time::MassaTime;
use num::rational::Ratio;
use tempfile::NamedTempFile;

/// Duplicated
pub fn generate_staking_keys_file(staking_keys: &[PrivateKey]) -> NamedTempFile {
    use std::io::prelude::*;
    let file_named = NamedTempFile::new().expect("cannot create temp file");
    serde_json::to_writer_pretty(file_named.as_file(), &staking_keys)
        .expect("unable to write ledger file");
    file_named
        .as_file()
        .seek(std::io::SeekFrom::Start(0))
        .expect("could not seek file");
    file_named
}

/// Duplicated
/// generate a named temporary JSON ledger file
pub fn generate_ledger_file(ledger_vec: &HashMap<Address, LedgerData>) -> NamedTempFile {
    use std::io::prelude::*;
    let ledger_file_named = NamedTempFile::new().expect("cannot create temp file");
    serde_json::to_writer_pretty(ledger_file_named.as_file(), &ledger_vec)
        .expect("unable to write ledger file");
    ledger_file_named
        .as_file()
        .seek(std::io::SeekFrom::Start(0))
        .expect("could not seek file");
    ledger_file_named
}

/// Duplicated
/// generate a named temporary JSON initial rolls file
pub fn generate_roll_counts_file(roll_counts: &RollCounts) -> NamedTempFile {
    use std::io::prelude::*;
    let roll_counts_file_named = NamedTempFile::new().expect("cannot create temp file");
    serde_json::to_writer_pretty(roll_counts_file_named.as_file(), &roll_counts.0)
        .expect("unable to write ledger file");
    roll_counts_file_named
        .as_file()
        .seek(std::io::SeekFrom::Start(0))
        .expect("could not seek file");
    roll_counts_file_named
}

/// Duplicated
/// generate a default named temporary JSON initial rolls file,
/// asuming two threads.
pub fn generate_default_roll_counts_file(stakers: Vec<PrivateKey>) -> NamedTempFile {
    let mut roll_counts = RollCounts::default();
    for key in stakers.iter() {
        let pub_key = derive_public_key(key);
        let address = Address::from_public_key(&pub_key);
        let update = RollUpdate {
            roll_purchases: 1,
            roll_sales: 0,
        };
        let mut updates = RollUpdates::default();
        updates.apply(&address, &update).unwrap();
        roll_counts.apply_updates(&updates).unwrap();
    }
    generate_roll_counts_file(&roll_counts)
}

/// Duplicated
/// TODO export this tools in a global config and tools module?
pub fn default_consensus_config(
    initial_ledger_path: &Path,
    roll_counts_path: &Path,
    staking_keys_path: &Path,
) -> ConsensusConfig {
    let genesis_key = generate_random_private_key();
    let thread_count: u8 = 2;
    let max_block_size: u32 = 3 * 1024 * 1024;
    let max_operations_per_block: u32 = 1024;
    let tempdir = tempfile::tempdir().expect("cannot create temp dir");

    // Init the serialization context with a default,
    // can be overwritten with a more specific one in the test.
    massa_models::init_serialization_context(massa_models::SerializationContext {
        max_block_operations: 1024,
        parent_count: 2,
        max_peer_list_length: 128,
        max_message_size: 3 * 1024 * 1024,
        max_block_size: 3 * 1024 * 1024,
        max_bootstrap_blocks: 100,
        max_bootstrap_cliques: 100,
        max_bootstrap_deps: 100,
        max_bootstrap_children: 100,
        max_ask_blocks_per_message: 10,
        max_operations_per_message: 1024,
        max_endorsements_per_message: 1024,
        max_bootstrap_message_size: 100000000,
        max_bootstrap_pos_entries: 1000,
        max_bootstrap_pos_cycles: 5,
        max_block_endorsements: 8,
    });

    ConsensusConfig {
        genesis_timestamp: MassaTime::now().unwrap(),
        thread_count,
        t0: 32000.into(),
        genesis_key,
        max_discarded_blocks: 10,
        future_block_processing_max_periods: 3,
        max_future_processing_blocks: 10,
        max_dependency_blocks: 10,
        delta_f0: 32,
        disable_block_creation: true,
        max_block_size,
        max_operations_per_block,
        max_operations_fill_attempts: 6,
        operation_validity_periods: 1,
        ledger_path: tempdir.path().to_path_buf(),
        ledger_cache_capacity: 1000000,
        ledger_flush_interval: Some(200.into()),
        ledger_reset_at_startup: true,
        block_reward: Amount::from_str("1").unwrap(),
        initial_ledger_path: initial_ledger_path.to_path_buf(),
        operation_batch_size: 100,
        initial_rolls_path: roll_counts_path.to_path_buf(),
        initial_draw_seed: "genesis".into(),
        periods_per_cycle: 100,
        pos_lookback_cycles: 2,
        pos_lock_cycles: 1,
        pos_draw_cached_cycles: 0,
        pos_miss_rate_deactivation_threshold: Ratio::new(1, 1),
        roll_price: Amount::default(),
        stats_timespan: 60000.into(),
        staking_keys_path: staking_keys_path.to_path_buf(),
        end_timestamp: None,
        max_send_wait: 500.into(),
        force_keep_final_periods: 0,
        endorsement_count: 0,
        block_db_prune_interval: 1000.into(),
        max_item_return_count: 1000,
    }
}