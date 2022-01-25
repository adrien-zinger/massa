// Copyright (c) 2021 MASSA LABS <info@massa.net>
#![feature(int_roundings)]

pub mod error;
mod roll_updates;
mod settings;
mod export_pos;
mod types;

use massa_models::{Operation, OperationType, Address};
use serde::{Deserialize, Serialize};

pub use roll_updates::{RollUpdates, RollUpdate};

mod roll_counts;
pub use roll_counts::RollCounts;

mod proof_of_stake;
pub use proof_of_stake::*;

pub use settings::ProofOfStakeConfig;
pub use types::POSBlock;
pub use export_pos::ExportProofOfStake;
use error::ProofOfStakeError;

mod thread_cycle_state;
pub use thread_cycle_state::ThreadCycleState;

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct RollCompensation(pub u64);

/// Roll specific method on operation
pub trait OperationRollInterface {
    /// get roll related modifications
    fn get_roll_updates(&self) -> Result<RollUpdates, ProofOfStakeError>;
}

impl OperationRollInterface for Operation {
    fn get_roll_updates(&self) -> Result<RollUpdates, ProofOfStakeError> {
        let mut res = RollUpdates::default();
        match self.content.op {
            OperationType::Transaction { .. } => {}
            OperationType::RollBuy { roll_count } => {
                res.apply(
                    &Address::from_public_key(&self.content.sender_public_key),
                    &RollUpdate {
                        roll_purchases: roll_count,
                        roll_sales: 0,
                    },
                )?;
            }
            OperationType::RollSell { roll_count } => {
                res.apply(
                    &Address::from_public_key(&self.content.sender_public_key),
                    &RollUpdate {
                        roll_purchases: 0,
                        roll_sales: roll_count,
                    },
                )?;
            }
            OperationType::ExecuteSC { .. } => {}
        }
        Ok(res)
    }
}
