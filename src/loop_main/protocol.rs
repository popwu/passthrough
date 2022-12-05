use serde::{Deserialize, Serialize};
use uuid::Uuid;

// 8 的结构
#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct NotReceived {
    pub id: Uuid,
    pub not_received: Vec<u64>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct UdpPackage {
    pub cmd: u8,
    pub buf: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct FileSend {
    // pub filename: Vec<u8>,
    // pub size: u64,
    pub id: Uuid,
    pub index: u64,
    pub buf: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct FileInfo {
    pub id: Uuid,
    pub filename: Vec<u8>,
    pub size: u64,
}