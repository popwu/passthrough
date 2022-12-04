use anyhow::Result;
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::time::{sleep, Duration};

use crate::GlobalConfig;
use crate::UdpPackage;

pub async fn keepalive(lsocket: &Arc<UdpSocket>, config: &Arc<GlobalConfig>) -> Result<()> {
    let who_am_i_byte = bincode::serialize(&UdpPackage {
        cmd: 3,
        buf: vec![0],
    })
    .unwrap();
    let echo_server: String;
    let sleep_time: u64;
    {
        echo_server = config.echo_server.clone();
        sleep_time = config.sleep_time;
    }
    loop {
        let _ret = lsocket.send_to(&who_am_i_byte, &echo_server).await?;
        let _ret = sleep(Duration::from_millis(sleep_time)).await;
    }
}
