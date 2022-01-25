use std::collections::VecDeque;

use massa_models::{SerializeCompact, ModelsError, DeserializeCompact, with_serialization_context, SerializeVarInt, DeserializeVarInt};
use serde::{Serialize, Deserialize};

use crate::{thread_cycle_state::ThreadCycleState, proof_of_stake::ProofOfStake};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportProofOfStake {
    /// Index by thread and cycle number
    pub cycle_states: Vec<VecDeque<ThreadCycleState>>,
}

impl From<&ProofOfStake> for ExportProofOfStake {
    fn from(pos: &ProofOfStake) -> Self {
        ExportProofOfStake {
            cycle_states: pos.cycle_states.clone(),
        }
    }
}

impl SerializeCompact for ExportProofOfStake {
    fn to_bytes_compact(&self) -> Result<Vec<u8>, ModelsError> {
        let mut res: Vec<u8> = Vec::new();
        for thread_lst in self.cycle_states.iter() {
            let cycle_count: u32 = thread_lst.len().try_into().map_err(|err| {
                ModelsError::SerializeError(format!(
                    "too many cycles when serializing ExportProofOfStake: {}",
                    err
                ))
            })?;
            res.extend(cycle_count.to_varint_bytes());
            for itm in thread_lst.iter() {
                res.extend(itm.to_bytes_compact()?);
            }
        }
        Ok(res)
    }
}

impl DeserializeCompact for ExportProofOfStake {
    fn from_bytes_compact(buffer: &[u8]) -> Result<(Self, usize), ModelsError> {
        let (thread_count, max_cycles) = with_serialization_context(|context| {
            (context.parent_count, context.max_bootstrap_pos_cycles)
        });
        let mut cursor = 0usize;

        let mut cycle_states = Vec::with_capacity(thread_count as usize);
        for thread in 0..thread_count {
            let (n_cycles, delta) = u32::from_varint_bytes(&buffer[cursor..])?;
            cursor += delta;
            if n_cycles == 0 || n_cycles > max_cycles {
                return Err(ModelsError::SerializeError(
                    "number of cycles invalid when deserializing ExportProofOfStake".into(),
                ));
            }
            cycle_states.push(VecDeque::with_capacity(n_cycles as usize));
            for _ in 0..n_cycles {
                let (thread_cycle_state, delta) =
                    ThreadCycleState::from_bytes_compact(&buffer[cursor..])?;
                cursor += delta;
                cycle_states[thread as usize].push_back(thread_cycle_state);
            }
        }
        Ok((ExportProofOfStake { cycle_states }, cursor))
    }
}