use anyhow::Result;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use crate::GlobalConfig;
use crate::UdpPackage;

pub async fn keepalive(lsocket: &Arc<UdpSocket>, config: &Arc<Mutex<GlobalConfig>>) -> Result<()> {
    let who_am_i_byte = bincode::serialize(&UdpPackage {
        cmd: 3,
        buf: vec![0],
    })
    .unwrap();
    loop {
        let _ret = lsocket
            .send_to(&who_am_i_byte, &config.lock().await.echo_server)
            .await?;
        let _ret = sleep(Duration::from_millis(config.lock().await.sleep_time)).await;
    }
}
