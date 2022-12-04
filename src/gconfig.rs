use std::net::SocketAddr;
use tokio::fs::File;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct GlobalConfig {
    pub action: String,
    pub self_addr: Mutex<Option<SocketAddr>>,
    pub fp: Mutex<Option<File>>,
    pub sleep_time: u64,
    pub filename: String,    
    pub echo_server: String,
    // pub tx: mpsc::Sender<ReceiveBuf>, 
    // pub rx: mpsc::Receiver<ReceiveBuf>,
}

#[derive(Debug)]
pub struct ReceiveBuf {
    pub from_addr: SocketAddr,
    pub buf: Vec<u8>,
}