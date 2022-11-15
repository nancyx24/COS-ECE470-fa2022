pub mod worker;

use log::info;

use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::time::Duration;
use ring::signature::{self, Ed25519KeyPair, UnparsedPublicKey, Signature, KeyPair, VerificationAlgorithm, EdDSAParameters};
use crate::types::transaction;

use std::thread;
use std::sync::{Arc, Mutex};

use crate::blockchain::Mempool;
use crate::types::address::Address;
use super::types::hash::Hashable;
use super::types::transaction::{Transaction, SignedTransaction};
use crate::types::key_pair;

enum ControlSignal {
    Start(u64), // the number controls the theta of interval between block generation
    Update, // update the block in mining, it may due to new blockchain tip or new transaction
    Exit,
}

enum OperatingState {
    Paused,
    Run(u64),
    ShutDown,
}

pub struct Context {
    // Channel for receiving control signal
    control_chan: Receiver<ControlSignal>,
    operating_state: OperatingState,
    finished_transaction_chan: Sender<SignedTransaction>,
    mempool: Arc<Mutex<Mempool>>,
}

#[derive(Clone)]
pub struct Handle {
    /// Channel for sending signal to the miner thread
    control_chan: Sender<ControlSignal>,
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

pub fn new(mempool: Arc<Mutex<Mempool>>) -> (Context, Handle, Receiver<SignedTransaction>) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();
    let (finished_tx_sender, finished_tx_receiver) = unbounded();

    let clone_mempool = Arc::clone(&mempool);

    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        finished_transaction_chan: finished_tx_sender,
        mempool: clone_mempool,
    };

    let handle = Handle {
        control_chan: signal_chan_sender,
    };

    (ctx, handle, finished_tx_receiver)
}

impl Context {
    pub fn start(mut self) {
        thread::Builder::new()
            .name("TxGenerator".to_string())
            .spawn(move || {
                self.tx_loop();
            })
            .unwrap();
        info!("TxGenerator initialized into paused mode");
    }

    fn tx_loop(&mut self) {
        // main mining loop
        loop {
            // check and react to control signals
            match self.operating_state {
                OperatingState::Paused => {
                    let signal = self.control_chan.recv().unwrap();
                    match signal {
                        ControlSignal::Exit => {
                            info!("TxGenerator shutting down");
                            self.operating_state = OperatingState::ShutDown;
                        }
                        ControlSignal::Start(i) => {
                            info!("TxGenerator starting in continuous mode with theta {}", i);
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
                                info!("TxGenerator shutting down");
                                self.operating_state = OperatingState::ShutDown;
                            }
                            ControlSignal::Start(i) => {
                                info!("TxGenerator starting in continuous mode with theta {}", i);
                                self.operating_state = OperatingState::Run(i);
                            }
                            ControlSignal::Update => {
                                unimplemented!()
                            }
                        };
                    }
                    Err(TryRecvError::Empty) => {}
                    Err(TryRecvError::Disconnected) => panic!("TxGenerator control channel detached"),
                },
            }
            if let OperatingState::ShutDown = self.operating_state {
                return;
            }

            // generate signed transaction
            let receiver = Address::from([0u8; 20]);
            let value = rand::random();
            let nonce = rand::random();
            let key_pair_rand = key_pair::random();
            let public_key = key_pair_rand.public_key().as_ref().to_vec();

            let transaction = Transaction::new(receiver, value, nonce);
            let sig = transaction::sign(&transaction, &key_pair_rand).as_ref().to_vec();
            let signed_transaction = SignedTransaction::new(transaction, sig, public_key);
            
            self.finished_transaction_chan.send(signed_transaction.clone()).expect("Send finished block error");
            println!("{}", nonce);
            // put into mempool
            {self.mempool.lock().unwrap().insert(signed_transaction.hash(), &signed_transaction)};
            println!("after insert into mempool");

            // END OF MY CODE
            if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = Duration::from_micros(i * 3000 as u64);
                    thread::sleep(interval);
                }
            }
        }
        }
    }
