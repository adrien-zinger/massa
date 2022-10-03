use crate::start_factory;

use super::TestFactory;
use massa_consensus_exports::test_exports::MockConsensusController;
use massa_factory_exports::{FactoryChannels, FactoryConfig};
use massa_models::{
    amount::Amount,
    operation::{Operation, OperationSerializer, OperationType},
    wrapped::WrappedContent,
};
use massa_pool_exports::test_exports::MockPoolController;
use massa_pos_exports::test_exports::MockSelectorController;
use massa_protocol_exports::test_exports::MockProtocolController;
use massa_signature::KeyPair;
use massa_storage::Storage;
use massa_wallet::{test_exports::create_test_wallet, Wallet};
use parking_lot::RwLock;
use std::{collections::HashMap, str::FromStr, sync::Arc};

/// Creates a basic empty block with the factory.
#[test]
#[ignore]
fn basic_creation() {
    let keypair = KeyPair::generate();
    let mut test_factory = TestFactory::new(&keypair);
    let (block_id, storage) = test_factory.get_next_created_block(None, None);
    assert_eq!(block_id, storage.read_blocks().get(&block_id).unwrap().id);
}

/// Creates a block with a roll buy operation in it.
#[test]
#[ignore]
fn basic_creation_with_operation() {
    let keypair = KeyPair::generate();
    let mut test_factory = TestFactory::new(&keypair);

    let content = Operation {
        fee: Amount::from_str("0.01").unwrap(),
        expire_period: 2,
        op: OperationType::RollBuy { roll_count: 1 },
    };
    let operation = Operation::new_wrapped(content, OperationSerializer::new(), &keypair).unwrap();
    let (block_id, storage) = test_factory.get_next_created_block(Some(vec![operation]), None);

    let block = storage.read_blocks().get(&block_id).unwrap().clone();
    for op_id in block.content.operations.iter() {
        storage.read_operations().get(op_id).unwrap();
    }
    assert_eq!(block.content.operations.len(), 1);
}

/// Creates a block with a multiple operations in it.
#[test]
#[ignore]
fn basic_creation_with_multiple_operations() {
    let keypair = KeyPair::generate();
    let mut test_factory = TestFactory::new(&keypair);

    let content = Operation {
        fee: Amount::from_str("0.01").unwrap(),
        expire_period: 2,
        op: OperationType::RollBuy { roll_count: 1 },
    };
    let operation = Operation::new_wrapped(content, OperationSerializer::new(), &keypair).unwrap();
    let (block_id, storage) =
        test_factory.get_next_created_block(Some(vec![operation.clone(), operation]), None);

    let block = storage.read_blocks().get(&block_id).unwrap().clone();
    for op_id in block.content.operations.iter() {
        storage.read_operations().get(op_id).unwrap();
    }
    assert_eq!(block.content.operations.len(), 2);
}

/// Creates a block with a multiple operations in it.
#[test]
#[ignore]
fn launch_factories() {
    let (selector, _) = MockSelectorController::new_with_receiver();
    let (_, consensus, _) = MockConsensusController::new_with_receiver();
    let (pool, _) = MockPoolController::new_with_receiver();
    let (_, protocol, _) = MockProtocolController::new();
    let mut manager = start_factory(
        FactoryConfig::default(),
        Arc::new(RwLock::new(create_test_wallet(None))),
        FactoryChannels {
            selector,
            consensus,
            pool,
            protocol,
            storage: Storage::create_root(),
        },
    );

    manager.stop()
}
