use crate::config::ExecutionConfig;
use crate::error::ExecutionError;
use crate::vm::VM;
use models::{Block, BlockHashMap};
use parking_lot::{Condvar, Mutex};
use std::sync::Arc;
use std::thread::{self, JoinHandle};
use tokio::sync::mpsc;

/// Commands sent to the `execution` component.
pub enum ExecutionCommand {
    /// The clique has changed,
    /// contains the blocks of the new blockclique
    /// and a list of blocks that became final
    BlockCliqueChanged {
        blockclique: BlockHashMap<Block>,
        finalized_blocks: BlockHashMap<Block>,
    },
}

// Events produced by the execution component.
pub enum ExecutionEvent {
    /// A coin transfer
    /// from the SCE ledger to the CSS ledger.
    TransferToConsensus,
}

/// Management commands sent to the `execution` component.
pub enum ExecutionManagementCommand {}

/// Execution queue.
#[derive(Clone, PartialEq)]
pub enum ExecutionQueue {
    Running,
    Shutdown,
    Stopped,
}

pub struct ExecutionWorker {
    /// Configuration
    _cfg: ExecutionConfig,
    /// Receiver of commands.
    controller_command_rx: mpsc::Receiver<ExecutionCommand>,
    /// Receiver of management commands.
    controller_manager_rx: mpsc::Receiver<ExecutionManagementCommand>,
    /// Sender of events.
    _event_sender: mpsc::UnboundedSender<ExecutionEvent>,
    /// Execution queue.
    execution_queue: Arc<(Mutex<ExecutionQueue>, Condvar)>,
    /// VM thread join handle.
    vm_join_handle: JoinHandle<()>,
}

impl ExecutionWorker {
    pub async fn new(
        cfg: ExecutionConfig,
        event_sender: mpsc::UnboundedSender<ExecutionEvent>,
        controller_command_rx: mpsc::Receiver<ExecutionCommand>,
        controller_manager_rx: mpsc::Receiver<ExecutionManagementCommand>,
    ) -> Result<ExecutionWorker, ExecutionError> {
        // Shared with the VM.
        let execution_queue = Arc::new((Mutex::new(ExecutionQueue::Running), Condvar::new()));
        let execution_queue_clone = execution_queue.clone();

        let vm = VM;

        let vm_join_handle = thread::spawn(move || {
            loop {
                let to_run = {
                    // Scoping the lock.
                    let &(ref lock, ref condvar) = &*execution_queue_clone;
                    let mut queue = lock.lock();

                    // Run until shutdown
                    while *queue != ExecutionQueue::Shutdown {
                        condvar.wait(&mut queue);

                        // Running normally.
                        match *queue {
                            ExecutionQueue::Running => {
                                // Return that which needs to be run...
                                ()
                            }
                            ExecutionQueue::Stopped => panic!("Unexpected execution queue state."),
                            _ => {
                                // Confirm shutdown
                                *queue = ExecutionQueue::Stopped;
                                condvar.notify_one();

                                // Dropping the lock.
                                return;
                            }
                        }
                    }
                };
                // Run stuff without holding the lock.
                vm.run(to_run);
            }
        });

        let worker = ExecutionWorker {
            _cfg: cfg,
            controller_command_rx,
            controller_manager_rx,
            _event_sender: event_sender,
            execution_queue,
            vm_join_handle,
        };

        Ok(worker)
    }

    pub async fn run_loop(mut self) -> Result<(), ExecutionError> {
        loop {
            tokio::select! {
                // Process management commands
                cmd = self.controller_manager_rx.recv() => {
                    match cmd {
                    None => break,
                    Some(_) => {}
                }}

                // Process commands
                Some(cmd) = self.controller_command_rx.recv() => self.process_command(cmd).await?,
            }
        }

        // Signal shutdown.
        let &(ref lock, ref condvar) = &*self.execution_queue;
        let mut queue = lock.lock();
        *queue = ExecutionQueue::Shutdown;
        condvar.notify_one();

        // Wait for shutdown confirmation.
        while *queue != ExecutionQueue::Stopped {
            condvar.wait(&mut queue);
        }

        // Join on the thread, once shutdown has been confirmed.
        self.vm_join_handle.join();

        // end loop
        Ok(())
    }

    /// Process a given command.
    ///
    /// # Argument
    /// * cmd: command to process
    async fn process_command(&mut self, cmd: ExecutionCommand) -> Result<(), ExecutionError> {
        match cmd {
            ExecutionCommand::BlockCliqueChanged {
                blockclique,
                finalized_blocks,
            } => {
                self.blockclique_changed(blockclique, finalized_blocks)?;
            }
        }
        Ok(())
    }

    fn blockclique_changed(
        &mut self,
        blockclique: BlockHashMap<Block>,
        finalized_blocks: BlockHashMap<Block>,
    ) -> Result<(), ExecutionError> {
        // TODO apply finalized blocks (note that they might not be SCE-final yet)

        // TODO apply new blockclique

        Ok(())
    }
}
