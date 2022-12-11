pub mod worker;

use log::info;

use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use std::time::Duration;
use ring::signature::{self, Ed25519KeyPair, UnparsedPublicKey, Signature, KeyPair, VerificationAlgorithm, EdDSAParameters};
use crate::types::transaction;

use std::thread;
use std::sync::{Arc, Mutex};

use crate::blockchain::Mempool;
use crate::blockchain::Blockchain;
use crate::types::address::Address;
use super::types::hash::Hashable;
use super::types::transaction::{Transaction, SignedTransaction};
use crate::types::key_pair;

use rand::Rng;

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
    blockchain: Arc<Mutex<Blockchain>>,
    mempool: Arc<Mutex<Mempool>>,
    accounts: Vec<Ed25519KeyPair>, // key is key_pair, value is (value, nonce)
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

pub fn new(mempool: Arc<Mutex<Mempool>>, key_pair: Ed25519KeyPair, blockchain: Arc<Mutex<Blockchain>>) -> (Context, Handle, Receiver<SignedTransaction>) {
    let (signal_chan_sender, signal_chan_receiver) = unbounded();
    let (finished_tx_sender, finished_tx_receiver) = unbounded();

    let clone_mempool = Arc::clone(&mempool);

    let mut accounts: Vec<Ed25519KeyPair> = Vec::new();
    accounts.push(key_pair);

    let ctx = Context {
        control_chan: signal_chan_receiver,
        operating_state: OperatingState::Paused,
        finished_transaction_chan: finished_tx_sender,
        blockchain,
        mempool: clone_mempool,
        accounts,
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

            // generate random receiver address
            let mut prob_rng = rand::thread_rng();
            let prob = prob_rng.gen_range(0.0..1.0);

            if prob < 0.1 {
                let prob_receiver_key_pair = key_pair::random();
                self.accounts.push(prob_receiver_key_pair); // ASK ABOUT THIS
            }

            // generate signed transaction

            // FOR SENDER
            for sender in &self.accounts {
                // println!("self.accounts not empty");
                let tip_hash = {self.blockchain.lock().unwrap().tip()};
                let tip_block = {self.blockchain.lock().unwrap().get_parent_block(tip_hash)};
                let state = tip_block.get_state();
                //println!("state empty?");
                // println!("{}", state.get_state().is_empty());

                // println!("state contains initial key pair?");
                // println!("{}", state.get_state().contains_key(&Address::from_public_key_bytes(self.accounts[0].public_key().as_ref())));

                let sender_address = Address::from_public_key_bytes(sender.public_key().as_ref());

                // println!("state contains sender_address?");
                // println!("{}", state.contains_key(sender_address));

                if state.contains_key(sender_address) {
                    // println!("state contains key");
                    let mut rng = rand::thread_rng();
                
                    let sender_nonce = state.get(sender_address.clone()).0;
                    let sender_balance = state.get(sender_address.clone()).1;

                    // set value and nonce of signed transaction
                    let value: u32;

                    // println!("sender_balance");
                    // println!("{}", sender_balance);

                    if sender_balance > 1 {
                        if sender_balance > 20 {
                            value = rng.gen_range(1..sender_balance / 10);
                        }
                        else {
                            value = rng.gen_range(1..sender_balance);
                        } 
                    }
                    else {
                        value = 0;
                    }
                    
                    let nonce = sender_nonce + 1;

                    // FOR RECEIVER

                    // generate receiver address
                    // if prob is less than 0.1, create another account
                    let receiver_index = rng.gen_range(0..self.accounts.len());
                    let receiver_key_pair = self.accounts.get(receiver_index).unwrap();
                    let receiver = Address::from_public_key_bytes(receiver_key_pair.public_key().as_ref());

                    let transaction = Transaction::new(receiver, value, nonce);
                    let sig = transaction::sign(&transaction, &sender).as_ref().to_vec();
                    let signed_transaction = SignedTransaction::new(transaction, sig, sender.public_key().as_ref().to_vec());
                    
                    // println!("transaction generated in txgen mod");
            
                    self.finished_transaction_chan.send(signed_transaction.clone()).expect("Send finished block error");
                    // put into mempool
                    {self.mempool.lock().unwrap().insert(signed_transaction.hash(), &signed_transaction)};

                    // println!("after insert into mempool");
                }
                
                
            }

            // END OF MY CODE
            if let OperatingState::Run(i) = self.operating_state {
                if i != 0 {
                    let interval = Duration::from_micros(i * 1000 as u64);
                    thread::sleep(interval);
                }
            }
        }
        }
    }
