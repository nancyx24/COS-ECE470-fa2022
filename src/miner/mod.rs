pub mod worker;

use log::info;

use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::time::{SystemTime, UNIX_EPOCH, Duration};

use std::thread;
use std::sync::{Arc, Mutex};

use crate::types::block::Block;
use crate::types::block;
use crate::miner::block::Content;
use crate::blockchain::Blockchain;
use crate::types::merkle::MerkleTree;
use super::types::hash::{Hashable, H256};
use std::convert::TryInto;
use super::types::transaction;
use hex_literal::hex;

enum ControlSignal {
    Start(u64), // the number controls the lambda of interval between block generation
    Update, // update the block in mining, it may due to new blockchain tip or new transaction
    Exit,
}

enum OperatingState {
    Paused,
    Run(u64),
    ShutDown,
}

pub struct Context {
    /// Channel for receiving control signal
    control_chan: Receiver<ControlSignal>,
    operating_state: OperatingState,
    finished_block_chan: Sender<Block>,
    blockchain: Arc<Mutex<Blockchain>>,
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
    blockchain: Arc<Mutex<Blockchain>>,
}

pub fn new() -> (Context, Handle, Receiver<Block>) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();
    let (finished_block_sender, finished_block_receiver) = unbounded();

    let blockchain_unprotected = Blockchain::new();
    let blockchain = Arc::new(Mutex::new(blockchain_unprotected));

    let clone_context = Arc::clone(&blockchain);

    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        finished_block_chan: finished_block_sender,
        blockchain: clone_context,
    };

    let clone_handle = Arc::clone(&blockchain);
    let handle = Handle {
        control_chan: signal_chan_sender,
        blockchain: clone_handle,
    };

    (ctx, handle, finished_block_receiver)
}

#[cfg(any(test,test_utilities))]
fn test_new() -> (Context, Handle, Receiver<Block>) {
    new()
}

impl Handle {
    pub fn exit(&self) {
        self.control_chan.send(ControlSignal::Exit).unwrap();
    }

    pub fn start(&self, lambda: u64) {
        self.control_chan
            .send(ControlSignal::Start(lambda))
            .unwrap();
    }

    pub fn update(&self) {
        self.control_chan.send(ControlSignal::Update).unwrap();
    }
}

impl Context {
    pub fn start(mut self) {
        thread::Builder::new()
            .name("miner".to_string())
            .spawn(move || {
                self.miner_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn miner_loop(&mut self) {
        let mut parent = self.blockchain.lock().unwrap().tip(); // MY CODE

        // main mining loop
        loop {
            // check and react to control signals
            match self.operating_state {
                OperatingState::Paused => {
                    let signal = self.control_chan.recv().unwrap();
                    match signal {
                        ControlSignal::Exit => {
                            info!("Miner shutting down");
                            self.operating_state = OperatingState::ShutDown;
                        }
                        ControlSignal::Start(i) => {
                            info!("Miner starting in continuous mode with lambda {}", i);
                            self.operating_state = OperatingState::Run(i);
                        }
                        ControlSignal::Update => {
                            // in paused state, don't need to update
                        }
                    };
                    continue;
                }
                OperatingState::ShutDown => {
                    return;
                }
                _ => match self.control_chan.try_recv() {
                    Ok(signal) => {
                        match signal {
                            ControlSignal::Exit => {
                                info!("Miner shutting down");
                                self.operating_state = OperatingState::ShutDown;
                            }
                            ControlSignal::Start(i) => {
                                info!("Miner starting in continuous mode with lambda {}", i);
                                self.operating_state = OperatingState::Run(i);
                            }
                            ControlSignal::Update => {
                                unimplemented!()
                            }
                        };
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("Miner control channel detached"),
                },
            }
            if let OperatingState::ShutDown = self.operating_state {
                return;
            }

            // TODO for student: actual mining, create a block
            // TODO for student: if block mining finished, you can have something like: self.finished_block_chan.send(block.clone()).expect("Send finished block error");
            
            // BEGINNING OF MY CODE
            
            // build a block
            let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(); // time now in milliseconds

            let parent_block = self.blockchain.lock().unwrap().get_parent_block(parent);
            let difficulty = parent_block.get_difficulty();
            
            // content is empty for now
            let content_data: Vec<transaction::SignedTransaction> = Vec::new();
            let data: Content = block::build_content(content_data);

            // get merkle root from content data
            let merkle_tree = MerkleTree::new(&data.get_content_data());
            let merkle_root = merkle_tree.root();

            // nonce, randomly generated
            let nonce = rand::random();

            // construct block
            let header = block::build_header(parent, nonce, difficulty, timestamp, merkle_root);
            let new_block = block::build_block(header, data);

            // check if successful
            if new_block.hash() <= difficulty {
                self.finished_block_chan.send(new_block.clone()).expect("Send finished block error");
                let mut blockchain = self.blockchain.lock().unwrap();
                blockchain.insert(&new_block);
                parent = self.blockchain.lock().unwrap().tip();
                let zero_parent = H256::from([0; 32]);
                if parent == zero_parent {
                    break;
                }
            }

            // // break if parent is genesis block
            // let zeros: [u8; 32] = [0; 32];
            // let zero_parent = H256::from(zeros);
            // if parent == zero_parent {
            //     self.finished_block_chan.send(new_block.clone()).expect("Send finished block error");
            //     break;
            // }

            // END OF MY CODE

            if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = Duration::from_micros(i as u64);
                    thread::sleep(interval);
                }
            }
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod test {
    use ntest::timeout;
    use crate::types::hash::Hashable;

    #[test]
    #[timeout(60000)]
    fn miner_three_block() {
        let (miner_ctx, miner_handle, finished_block_chan) = super::test_new();
        miner_ctx.start();
        miner_handle.start(0);
        let mut block_prev = finished_block_chan.recv().unwrap();
        for _ in 0..2 {
            let block_next = finished_block_chan.recv().unwrap();
            assert_eq!(block_prev.hash(), block_next.get_parent());
            block_prev = block_next;
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
