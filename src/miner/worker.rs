use crossbeam::channel::{unbounded, Receiver, Sender, TryRecvError};
use log::{debug, info};
use crate::types::block::Block;
use crate::network::server::Handle as ServerHandle;
use std::thread;
use crate::blockchain::Blockchain;
use std::sync::{Arc, Mutex};
use crate::types::hash::{H256, Hashable};
use crate::network::message::Message;

#[derive(Clone)]
pub struct Worker {
    server: ServerHandle,
    finished_block_chan: Receiver<Block>,
    blockchain: Arc<Mutex<Blockchain>>,
}

impl Worker {
    pub fn new(
        server: &ServerHandle,
        finished_block_chan: Receiver<Block>,
        blockchain: Arc<Mutex<Blockchain>>,
    ) -> Self {
        Self {
            server: server.clone(),
            finished_block_chan,
            blockchain,
        }
    }

    pub fn start(self) {
        thread::Builder::new()
            .name("miner-worker".to_string())
            .spawn(move || {
                self.worker_loop();
            })
            .unwrap();
        info!("Miner initialized into paused mode");
    }

    fn worker_loop(&self) {
        loop {
            let _block = self.finished_block_chan.recv().expect("Receive finished block error");
            
            // TODO for student: insert this finished block to blockchain, and broadcast this block hash
            let mut blockchain = self.blockchain.lock().unwrap();
            blockchain.insert(&_block);
            
            // if successful, broadcast message NewBlockHashes
            let mut message: Vec<H256> = Vec::new();
            message.push(_block.hash());
            self.server.broadcast(Message::NewBlockHashes(message));
        }
    }
}
