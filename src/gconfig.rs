use std::net::SocketAddr;
use std::collections::HashMap;
use uuid::Uuid;
use tokio::fs::File;
use tokio::sync::Mutex;

#[derive(Debug)]
pub struct SendFileTask {
    pub filename: Vec<u8>,
    pub size: u64,
    pub remote_addr: SocketAddr,
    pub fp: File,
}

#[derive(Debug)]
pub struct ReceiveFileTask {
    // pub filename: Vec<u8>,
    pub from_addr: SocketAddr,
    // pub is_ok: bool,
    pub received: Vec<u64>,
    pub fp: File,
}

#[derive(Debug)]
pub enum Task {
    SendFile(SendFileTask),
    ReceiveFile(ReceiveFileTask),
}

#[derive(Debug)]
pub struct GlobalConfig {
    pub action: String,
    pub self_addr: Mutex<Option<SocketAddr>>,
    pub fp: Mutex<Option<File>>,
    pub sleep_time: u64,
    pub filename: String,
    pub echo_server: String,
    pub tasks: Mutex<HashMap<Uuid, Task>>,
    // pub tx: mpsc::Sender<ReceiveBuf>, 
    // pub rx: mpsc::Receiver<ReceiveBuf>,
}

#[derive(Debug)]
pub struct ReceiveBuf {
    pub from_addr: SocketAddr,
    pub buf: Vec<u8>,
}