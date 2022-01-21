#![feature(async_closure)]
#![feature(bool_to_option)]
#![feature(hash_drain_filter)]
#![feature(map_first_last)]
#![feature(int_roundings)]

extern crate tempfile;

use std::{collections::HashMap, sync::{Arc, Mutex}};

use massa_consensus::{ConsensusConfig, ConsensusChannels, start_consensus_controller};
use massa_hash::hash::Hash;
use massa_models::{Address, OperationType, Operation, OperationContent, Amount, SerializeCompact, OperationId};
use massa_pool::PoolCommand;
use massa_signature::{generate_random_private_key, derive_public_key, sign, PrivateKey, PublicKey};

mod tools;
// TODO use definitively a module for all that stuff...
mod mock_execution_controller;
mod mock_protocol_controller;
mod mock_pool_controller;
use mock_execution_controller::MockExecutionController;
use mock_pool_controller::MockPoolController;
use mock_protocol_controller::MockProtocolController;
use rand::Rng;
use tools::*;
// TODO we should use a const config manager as cornetto
pub const THREAD_COUNT: u8 = 32;
pub const ROLL_PRICE: Amount = Amount::from_raw(100);

lazy_static::lazy_static! {
    static ref BENCH: ConsensusConfig = {
        let ledger = HashMap::new();
        let ledger_file = generate_ledger_file(&ledger);
        let privkey = generate_random_private_key();
        let staking_file = generate_staking_keys_file(&[privkey]);
        let roll_counts_file = generate_default_roll_counts_file(vec![privkey]);
        tools::default_consensus_config(
            ledger_file.path(),
            roll_counts_file.path(),
            staking_file.path(),
        )
    };
}

// This is a struct that tells Criterion.rs to use the "futures" crate's current-thread executor
fn build_n_random_addresse(n: usize) -> (Vec<Address>, Vec<PrivateKey>, Vec<PublicKey>) {
    let mut addresses = vec![];
    let mut privkeys = vec![];
    let mut pubkeys = vec![];
    for _ in 0..n {
        let privkey = generate_random_private_key();
        let pubkey = derive_public_key(&privkey);
        addresses.push(Address::from_public_key(&pubkey));
        privkeys.push(privkey);
        pubkeys.push(pubkey);
    };
    (addresses, privkeys, pubkeys)
}

pub fn create_roll_operation(
    priv_key: PrivateKey,
    ty: OperationType,
) -> Operation {
    let sender_public_key = derive_public_key(&priv_key);
    let content = OperationContent {
        sender_public_key,
        fee: Amount::from_raw(0),
        expire_period: 90,
        op: ty,
    };
    let hash = Hash::compute_from(&content.to_bytes_compact().unwrap());
    let signature = sign(&hash, &priv_key).unwrap();
    Operation { content, signature }
}

// Half-duplicated
fn build_n_random_roll_operation(n: usize, privkeys: &[PrivateKey], balances: &mut [Amount], rolls: &mut [u64]) -> Vec<(OperationId, Operation, u64)> {
    let mut th_rng = rand::thread_rng();
    let mut res = vec![];
    for _ in 0..n {
        let addr = th_rng.gen_range(0..privkeys.len());
        let ty = if rolls[addr] == 0 || th_rng.gen::<bool>() {
            let roll_count = th_rng.gen_range(1..(balances[addr].to_raw() / ROLL_PRICE.to_raw()));
            rolls[addr] += roll_count;
            OperationType::RollBuy { roll_count }
        } else {
            let roll_count = th_rng.gen_range(1..rolls[addr]);
            rolls[addr] -= roll_count;
            OperationType::RollSell { roll_count }
        };
        let op = create_roll_operation(privkeys[addr], ty);
        res.push((op.get_operation_id().unwrap(), op, 10 /* TODO please what is this 10? the validity? The âge du capitaine? */));
    }
    res
}

#[tokio::main]
async fn main() {
    // mock protocol & pool
    let (_, protocol_command_sender, protocol_event_receiver) =
        MockProtocolController::new();
    let (mut pool_controller, pool_command_sender) = MockPoolController::new();
    let (mut _execution_controller, execution_command_sender, execution_event_receiver) =
        MockExecutionController::new();

    let cfg = BENCH.clone();
    const ADDRESS_NUMBER: usize = 10;
    const OPERATION_NUMBER: usize = 10;
    let (_, privkeys, _) = build_n_random_addresse(ADDRESS_NUMBER);

    let (_, _consensus_event_receiver, _consensus_manager) =
    start_consensus_controller(
        cfg.clone(),
        ConsensusChannels {
            execution_command_sender,
            execution_event_receiver,
            protocol_command_sender: protocol_command_sender.clone(),
            protocol_event_receiver,
            pool_command_sender,
        },
        None,
        None,
        0,
    )
    .await
    .expect("could not start consensus controller");

    // TODO spawn polution futures after benchmarking the full node with tokio tracing
    for _ in 0..10 {
        let rolls = Arc::new(Mutex::new([0u64; ADDRESS_NUMBER]));
        let balances = Arc::new(Mutex::new([Amount::from_raw(10000); ADDRESS_NUMBER]));
        pool_controller
            .wait_command(300.into(), |cmd| match cmd {
                PoolCommand::GetOperationBatch {
                    response_tx,
                    ..
                } => {
                    let mut rolls = *rolls.lock().unwrap();
                    let mut balances = *balances.lock().unwrap();
                    response_tx
                        .send(build_n_random_roll_operation(OPERATION_NUMBER, &privkeys, &mut balances, &mut rolls))
                        .unwrap();
                    Some(())
                }
                PoolCommand::GetEndorsements { response_tx, .. } => {
                    response_tx.send(Vec::new()).unwrap();
                    None
                }
                _ => None,
            })
            .await
            .expect("timeout while waiting for 1st operation batch request");
    }
}
