use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct UdpPackage {
    pub cmd: u8,
    pub buf: Vec<u8>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub struct FileSend {
    // pub filename: Vec<u8>,
    pub size: u64,
    pub index: u64,
    pub buf: Vec<u8>,
}