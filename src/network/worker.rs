use super::message::Message;
use super::peer;
use super::server::Handle as ServerHandle;
use crate::types::hash::{H256, Hashable};
use std::sync::{Arc, Mutex};
use crate::types::block::Block;
use crate::blockchain::Blockchain;
use std::collections::HashMap;

use log::{debug, warn, error};

use std::thread;

#[cfg(any(test,test_utilities))]
use super::peer::TestReceiver as PeerTestReceiver;
#[cfg(any(test,test_utilities))]
use super::server::TestReceiver as ServerTestReceiver;
#[derive(Clone)]
pub struct Worker {
    msg_chan: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
    num_worker: usize,
    server: ServerHandle,
    blockchain: Arc<Mutex<Blockchain>>,
}


impl Worker {
    pub fn new(
        num_worker: usize,
        msg_src: smol::channel::Receiver<(Vec<u8>, peer::Handle)>,
        server: &ServerHandle,
        blockchain: &Arc<Mutex<Blockchain>>,
    ) -> Self {
        Self {
            msg_chan: msg_src,
            num_worker,
            server: server.clone(),
            blockchain: blockchain.clone(),
        }
    }

    pub fn start(self) {
        let num_worker = self.num_worker;
        for i in 0..num_worker {
            let cloned = self.clone();
            thread::spawn(move || {
                cloned.worker_loop();
                warn!("Worker thread {} exited", i);
            });
        }
    }

    fn worker_loop(&self) {
        loop {
            let result = smol::block_on(self.msg_chan.recv());
            if let Err(e) = result {
                error!("network worker terminated {}", e);
                break;
            }
            let msg = result.unwrap();
            let (msg, mut peer) = msg;
            let msg: Message = bincode::deserialize(&msg).unwrap();
            match msg {
                Message::Ping(nonce) => {
                    debug!("Ping: {}", nonce);
                    peer.write(Message::Pong(nonce.to_string()));
                }
                Message::Pong(nonce) => {
                    debug!("Pong: {}", nonce);
                }
                Message::NewBlockHashes(nonce) => { // 
                    // let mut blocks_needed: Vec<H256> = Vec::new();

                    // push blocks not in blockchain
                    for el in nonce {
                        let mut blocks_needed: Vec<H256> = Vec::new();

                        if !{self.blockchain.lock().unwrap().is_present(el)} {
                            blocks_needed.push(el);
                        }

                        peer.write(Message::GetBlocks(blocks_needed));

                        // println!("inside message::newBlockHashes");
                        // println!("{}", el.hash());
                    }
                }
                Message::GetBlocks(nonce) => {
                    // push blocks in blockchain
                    for el in nonce {
                        let mut blocks_have: Vec<Block> = Vec::new();

                        if self.blockchain.lock().unwrap().is_present(el) {
                            blocks_have.push(self.blockchain.lock().unwrap().get_parent_block(el));
                        }
                        // println!("inside message::getBlocks");
                        // println!("{}", el.hash());

                        // reply with the have blocks
                        peer.write(Message::Blocks(blocks_have));
                    }
                }
                Message::Blocks(nonce) => {
                    // orphan buffer
                    // key is parent of block, value is block
                    let mut orphan_buffer: HashMap<H256, Block> = HashMap::new();

                    for el in nonce {
                        let mut new_blocks: Vec<H256> = Vec::new();

                        if !{self.blockchain.lock().unwrap().is_present(el.hash())} {
                            // PoW check
                            if el.hash() <= el.get_difficulty() {
                                // parent check
                                if self.blockchain.lock().unwrap().is_present(el.get_parent()) {
                                    // check difficulty in block header consistent with view
                                    if el.get_difficulty() == self.blockchain.lock().unwrap().get_parent_block(el.get_parent()).get_difficulty() {
                                        // insert into blockchain
                                        {self.blockchain.lock().unwrap().insert(&el)};
                                        // println!("inserted block -- network worker");
                                        // println!("{}", el.hash());
                                        // insert into vector of new blocks
                                        new_blocks.push(el.hash());

                                        // orphan block handler
                                        let mut count = el.clone();
                                        while orphan_buffer.contains_key(&count.hash()) {
                                            // process orphan block
                                            let orphan = orphan_buffer.get(&count.hash()).unwrap();
                                            {self.blockchain.lock().unwrap().insert(&orphan)};
                                            new_blocks.push(orphan.hash());

                                            // update counter
                                            count = orphan.clone();
                                        }
                                    } 
                                }
                                else {
                                    // add block to orphan buffer
                                    orphan_buffer.insert(el.get_parent(), el.clone());

                                    // send getBlocks message with parent hash
                                    let mut to_send: Vec<H256> = Vec::new();
                                    to_send.push(el.clone().get_parent().hash());
                                    peer.write(Message::GetBlocks(to_send));
                                }
                                
                            }
                        }
                        // broadcast new blocks
                        self.server.broadcast(Message::NewBlockHashes(new_blocks));
                    }
                } // all other cases
                _ => unimplemented!(),
            }
        }
    }
}

#[cfg(any(test,test_utilities))]
struct TestMsgSender {
    s: smol::channel::Sender<(Vec<u8>, peer::Handle)>
}
#[cfg(any(test,test_utilities))]
impl TestMsgSender {
    fn new() -> (TestMsgSender, smol::channel::Receiver<(Vec<u8>, peer::Handle)>) {
        let (s,r) = smol::channel::unbounded();
        (TestMsgSender {s}, r)
    }

    fn send(&self, msg: Message) -> PeerTestReceiver {
        let bytes = bincode::serialize(&msg).unwrap();
        let (handle, r) = peer::Handle::test_handle();
        smol::block_on(self.s.send((bytes, handle))).unwrap();
        r
    }
}
#[cfg(any(test,test_utilities))]
/// returns two structs used by tests, and an ordered vector of hashes of all blocks in the blockchain
fn generate_test_worker_and_start() -> (TestMsgSender, ServerTestReceiver, Vec<H256>) {
    let (server, server_receiver) = ServerHandle::new_for_test();
    let (test_msg_sender, msg_chan) = TestMsgSender::new();
    let blockchain = Arc::new(Mutex::new(Blockchain::new()));
    let worker = Worker::new(1, msg_chan, &server, &blockchain);

    // blockchain longest chain
    let longest = worker.blockchain.lock().unwrap().all_blocks_in_longest_chain();

    worker.start();

    (test_msg_sender, server_receiver, longest)
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. BEFORE TEST

#[cfg(test)]
mod test {
    use ntest::timeout;
    use crate::types::block::generate_random_block;
    use crate::types::hash::Hashable;

    use super::super::message::Message;
    use super::generate_test_worker_and_start;

    #[test]
    #[timeout(60000)]
    fn reply_new_block_hashes() {
        let (test_msg_sender, _server_receiver, v) = generate_test_worker_and_start();
        let random_block = generate_random_block(v.last().unwrap());
        let mut peer_receiver = test_msg_sender.send(Message::NewBlockHashes(vec![random_block.hash()]));
        let reply = peer_receiver.recv();
        if let Message::GetBlocks(v) = reply {
            assert_eq!(v, vec![random_block.hash()]);
        } else {
            panic!();
        }
    }
    #[test]
    #[timeout(60000)]
    fn reply_get_blocks() {
        let (test_msg_sender, _server_receiver, v) = generate_test_worker_and_start();
        let h = v.last().unwrap().clone();
        let mut peer_receiver = test_msg_sender.send(Message::GetBlocks(vec![h.clone()]));
        let reply = peer_receiver.recv();
        if let Message::Blocks(v) = reply {
            assert_eq!(1, v.len());
            assert_eq!(h, v[0].hash())
        } else {
            panic!();
        }
    }
    #[test]
    #[timeout(60000)]
    fn reply_blocks() {
        let (test_msg_sender, server_receiver, v) = generate_test_worker_and_start();
        let random_block = generate_random_block(v.last().unwrap());
        let mut _peer_receiver = test_msg_sender.send(Message::Blocks(vec![random_block.clone()]));
        let reply = server_receiver.recv().unwrap();
        if let Message::NewBlockHashes(v) = reply {
            assert_eq!(v, vec![random_block.hash()]);
        } else {
            panic!();
        }
    }
}

// DO NOT CHANGE THIS COMMENT, IT IS FOR AUTOGRADER. AFTER TEST
