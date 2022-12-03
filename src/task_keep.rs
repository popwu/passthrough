use anyhow::Result;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use crate::GlobalConfig;
use crate::UdpPackage;

pub async fn keepalive(lsocket: Arc<UdpSocket>, config: Arc<Mutex<GlobalConfig>>) -> Result<()> {
    let who_am_i = UdpPackage{
        cmd: 3,
        buf: vec![0],
    };
    let who_am_i_byte = bincode::serialize(&who_am_i).unwrap();
    loop {
        
        let _ret = lsocket.send_to(&who_am_i_byte, "101.43.36.196:8882").await?;
        let sleep_time: u64;
        {
            let config = config.lock().await;
            sleep_time = config.sleep_time;
        }
        sleep(Duration::from_millis(sleep_time)).await;
    }
}
