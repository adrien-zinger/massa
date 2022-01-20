use std::collections::HashMap;

use criterion::{Criterion, Bencher};
use criterion::{criterion_group, criterion_main};
use massa_consensus::ConsensusConfig;
use massa_models::Address;
use massa_signature::{generate_random_private_key, derive_public_key};

mod tools;
// TODO use definitively a module for all that stuff...
mod mock_execution_controller;
mod mock_protocol_controller;
mod mock_pool_controller;
use mock_execution_controller::MockExecutionController;
use mock_pool_controller::MockPoolController;
use mock_protocol_controller::MockProtocolController;
use tools::*;
// TODO we should use a const config manager as cornetto
pub const THREAD_COUNT: u8 = 32;

lazy_static::lazy_static! {
    static ref BENCH: ConsensusConfig = {
        let ledger = HashMap::new();
        let ledger_file = generate_ledger_file(&ledger);
        let privkey = generate_random_private_key();
        let staking_file = generate_staking_keys_file(&vec![privkey]);
        let roll_counts_file = generate_default_roll_counts_file(vec![privkey]);
        tools::default_consensus_config(
            ledger_file.path(),
            roll_counts_file.path(),
            staking_file.path(),
        )
    };
}



// This is a struct that tells Criterion.rs to use the "futures" crate's current-thread executor
fn build_n_random_addresse(n: usize) -> Vec<Address> {
    let mut ret = vec![];
    for _ in 0..n {
        let privkey = generate_random_private_key();
        let pubkey = derive_public_key(&privkey);
        ret.push(Address::from_public_key(&pubkey))
    };
    ret
}

fn create_random_rolls() {
    let cfg = BENCH.clone();

}

/*
fn create_random_rolls() {
    let thread_count = 2;
    // define addresses use for the test
    // addresses 1 and 2 both in thread 0
    let mut priv_1 = generate_random_private_key();
    let mut pubkey_1 = derive_public_key(&priv_1);
    let mut address_1 = Address::from_public_key(&pubkey_1);
    while 0 != address_1.get_thread(thread_count) {
        priv_1 = generate_random_private_key();
        pubkey_1 = derive_public_key(&priv_1);
        address_1 = Address::from_public_key(&pubkey_1);
    }
    assert_eq!(0, address_1.get_thread(thread_count));

    let mut priv_2 = generate_random_private_key();
    let mut pubkey_2 = derive_public_key(&priv_2);
    let mut address_2 = Address::from_public_key(&pubkey_2);
    while 0 != address_2.get_thread(thread_count) {
        priv_2 = generate_random_private_key();
        pubkey_2 = derive_public_key(&priv_2);
        address_2 = Address::from_public_key(&pubkey_2);
    }
    assert_eq!(0, address_2.get_thread(thread_count));

    let mut ledger = HashMap::new();
    ledger.insert(
        address_2,
        LedgerData::new(Amount::from_str("10000").unwrap()),
    );
    let ledger_file = generate_ledger_file(&ledger);

    let staking_file = tools::generate_staking_keys_file(&vec![priv_1]);
    let roll_counts_file = tools::generate_default_roll_counts_file(vec![priv_1]);
    let mut cfg = tools::default_consensus_config(
        ledger_file.path(),
        roll_counts_file.path(),
        staking_file.path(),
    );
    cfg.periods_per_cycle = 2;
    cfg.pos_lookback_cycles = 2;
    cfg.pos_lock_cycles = 1;
    cfg.t0 = 500.into();
    cfg.delta_f0 = 3;
    cfg.disable_block_creation = false;
    cfg.thread_count = thread_count;
    cfg.operation_validity_periods = 10;
    cfg.operation_batch_size = 500;
    cfg.max_operations_per_block = 5000;
    cfg.max_block_size = 500;
    cfg.block_reward = Amount::default();
    cfg.roll_price = Amount::from_str("1000").unwrap();
    cfg.operation_validity_periods = 100;

    // mock protocol & pool
    let (mut protocol_controller, protocol_command_sender, protocol_event_receiver) =
        MockProtocolController::new();
    let (mut pool_controller, pool_command_sender) = MockPoolController::new();
    let (mut _execution_controller, execution_command_sender, execution_event_receiver) =
        MockExecutionController::new();

    let init_time: MassaTime = 1000.into();
    cfg.genesis_timestamp = MassaTime::now().unwrap().saturating_add(init_time);

    // launch consensus controller
    let (consensus_command_sender, _consensus_event_receiver, _consensus_manager) =
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

    // operations
    let rb_a2_r1 = create_roll_buy(priv_2, 1, 90, 0);
    let rs_a2_r1 = create_roll_sell(priv_2, 1, 90, 0);

    let mut addresses = Set::<Address>::default();
    addresses.insert(address_2);
    let addresses = addresses;

    // wait for first slot
    pool_controller
        .wait_command(
            cfg.t0.saturating_mul(2).saturating_add(init_time),
            |cmd| match cmd {
                PoolCommand::UpdateCurrentSlot(s) => {
                    if s == Slot::new(1, 0) {
                        Some(())
                    } else {
                        None
                    }
                }
                PoolCommand::GetEndorsements { response_tx, .. } => {
                    response_tx.send(Vec::new()).unwrap();
                    None
                }
                _ => None,
            },
        )
        .await
        .expect("timeout while waiting for slot");

    // cycle 0

    // respond to first pool batch command
    pool_controller
        .wait_command(300.into(), |cmd| match cmd {
            PoolCommand::GetOperationBatch {
                response_tx,
                target_slot,
                ..
            } => {
                assert_eq!(target_slot, Slot::new(1, 0));
                response_tx
                    .send(vec![(
                        rb_a2_r1.clone().get_operation_id().unwrap(),
                        rb_a2_r1.clone(),
                        10,
                    )])
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

    // wait for block
    let (_block_id, block) = protocol_controller
        .wait_command(500.into(), |cmd| match cmd {
            ProtocolCommand::IntegratedBlock {
                block_id, block, ..
            } => Some((block_id, block)),
            _ => None,
        })
        .await
        .expect("timeout while waiting for block");

    // assert it's the expected block
    assert_eq!(block.header.content.slot, Slot::new(1, 0));
    assert_eq!(block.operations.len(), 1);
    assert_eq!(
        block.operations[0].get_operation_id().unwrap(),
        rb_a2_r1.clone().get_operation_id().unwrap()
    );

    let addr_state = consensus_command_sender
        .get_addresses_info(addresses.clone())
        .await
        .unwrap()
        .get(&address_2)
        .unwrap()
        .clone();
    assert_eq!(addr_state.rolls.active_rolls, 0);
    assert_eq!(addr_state.rolls.final_rolls, 0);
    assert_eq!(addr_state.rolls.candidate_rolls, 1);

    let balance = consensus_command_sender
        .get_addresses_info(addresses.clone())
        .await
        .unwrap()
        .get(&address_2)
        .unwrap()
        .ledger_info
        .candidate_ledger_info
        .balance;
    assert_eq!(balance, Amount::from_str("9000").unwrap());

    wait_pool_slot(&mut pool_controller, cfg.t0, 1, 1).await;
    // slot 1,1
    pool_controller
        .wait_command(300.into(), |cmd| match cmd {
            PoolCommand::GetOperationBatch {
                response_tx,
                target_slot,
                ..
            } => {
                assert_eq!(target_slot, Slot::new(1, 1));
                response_tx.send(vec![]).unwrap();
                Some(())
            }
            PoolCommand::GetEndorsements { response_tx, .. } => {
                response_tx.send(Vec::new()).unwrap();
                None
            }
            _ => None,
        })
        .await
        .expect("timeout while waiting for operation batch request");

    // wait for block
    let (_block_id, block) = protocol_controller
        .wait_command(500.into(), |cmd| match cmd {
            ProtocolCommand::IntegratedBlock {
                block_id, block, ..
            } => Some((block_id, block)),
            _ => None,
        })
        .await
        .expect("timeout while waiting for block");

    // assert it's the expected block
    assert_eq!(block.header.content.slot, Slot::new(1, 1));
    assert!(block.operations.is_empty());

    // cycle 1

    pool_controller
        .wait_command(300.into(), |cmd| match cmd {
            PoolCommand::GetOperationBatch {
                response_tx,
                target_slot,
                ..
            } => {
                assert_eq!(target_slot, Slot::new(2, 0));
                response_tx
                    .send(vec![(
                        rs_a2_r1.clone().get_operation_id().unwrap(),
                        rs_a2_r1.clone(),
                        10,
                    )])
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

    // wait for block
    let (_block_id, block) = protocol_controller
        .wait_command(500.into(), |cmd| match cmd {
            ProtocolCommand::IntegratedBlock {
                block_id, block, ..
            } => Some((block_id, block)),
            _ => None,
        })
        .await
        .expect("timeout while waiting for block");

    // assert it's the expected block
    assert_eq!(block.header.content.slot, Slot::new(2, 0));
    assert_eq!(block.operations.len(), 1);
    assert_eq!(
        block.operations[0].get_operation_id().unwrap(),
        rs_a2_r1.clone().get_operation_id().unwrap()
    );

    let addr_state = consensus_command_sender
        .get_addresses_info(addresses.clone())
        .await
        .unwrap()
        .get(&address_2)
        .unwrap()
        .clone();
    assert_eq!(addr_state.rolls.active_rolls, 0);
    assert_eq!(addr_state.rolls.final_rolls, 0);
    assert_eq!(addr_state.rolls.candidate_rolls, 0);
    let balance = addr_state.ledger_info.candidate_ledger_info.balance;
    assert_eq!(balance, Amount::from_str("9000").unwrap());
}
*/

fn from_elem(c: &mut Criterion) {
    let runner = tokio::runtime::Runtime::new().unwrap();
    // mock protocol & pool
    let (mut protocol_controller, protocol_command_sender, protocol_event_receiver) =
        MockProtocolController::new();
    let (mut pool_controller, pool_command_sender) = MockPoolController::new();
    let (mut _execution_controller, execution_command_sender, execution_event_receiver) =
        MockExecutionController::new();

    c.bench_function("iter", move |b: &mut Bencher| {
        b.to_async(&runner).iter( || async {
            // async tokio
        })
    });
}

criterion_group!(benches, from_elem);
criterion_main!(benches);
