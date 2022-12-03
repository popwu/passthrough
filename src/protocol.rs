use serde::{Deserialize, Serialize};

// 1 server -> node ip
// 2 node -> node file_split
// 3 node -> server ip
// 4 node -> server bridge_node
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