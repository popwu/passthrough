use std::net::SocketAddr;
use tokio::fs::File;

pub struct GlobalConfig {
    pub action: String,
    pub self_addr: Option<SocketAddr>,
    pub sleep_time: u64,
    pub filename: String,
    pub fp: Option<File>,
}