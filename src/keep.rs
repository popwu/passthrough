use anyhow::Result;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use crate::GlobalConfig;

pub async fn keepalive(lsocket: Arc<UdpSocket>, config: Arc<Mutex<GlobalConfig>>) -> Result<()> {
    loop {
        let _ret = lsocket.send_to("h".as_bytes(), "101.43.36.196:8882").await;
        let mut sleep_time: u64;
        {
            let mut config = config.lock().await;
            // config.sleep_time = config.sleep_time + 1000;
            sleep_time = config.sleep_time;
        }
        sleep(Duration::from_millis(sleep_time)).await;
    }
}
