use std::net::SocketAddr;
use tokio::fs::File;

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
pub struct ReceiveBuf {
    pub from_addr: SocketAddr,
    pub buf: Vec<u8>,
}