use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use log::{debug, info};
use crate::types::block::Block;
use crate::network::server::Handle as ServerHandle;
use std::thread;
use crate::blockchain::{Blockchain, Mempool};
use std::sync::{Arc, Mutex};
use crate::types::hash::{H256, Hashable};
use crate::network::message::Message;
use crate::types::transaction::SignedTransaction;

#[derive(Clone)]
pub struct Worker {
    server: ServerHandle,
    finished_transaction_chan: Receiver<SignedTransaction>,
    mempool: Arc<Mutex<Mempool>>,
}

impl Worker {
    pub fn new(
        server: &ServerHandle,
        finished_transaction_chan: Receiver<SignedTransaction>,
        mempool: Arc<Mutex<Mempool>>,
    ) -> Self {
        Self {
            server: server.clone(),
            finished_transaction_chan,
            mempool,
        }
    }

    pub fn start(self) {
        thread::Builder::new()
            .name("txgen-worker".to_string())
            .spawn(move || {
                self.worker_loop();
            })
            .unwrap();
        info!("txgen initialized into paused mode");
    }

    fn worker_loop(&self) {
        loop {
            let _tx = self.finished_transaction_chan.recv().expect("Receive finished tx error");
            
            // TODO for student: insert this finished block to mempool, and broadcast this block hash
            {self.mempool.lock().unwrap().insert(_tx.hash(), &_tx)};
            // println!("{}", _tx.hash());
            // println!("inside txgen worker");

            // if successful, broadcast message NewTransactionHashes
            let mut message: Vec<H256> = Vec::new();
            message.push(_tx.hash());
            self.server.broadcast(Message::NewTransactionHashes(message));
            // println!("broadcast in txgen worker");
        }
    }
}
