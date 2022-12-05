use anyhow::{Ok, Result};
use uuid::Uuid;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::fs::{metadata, File};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use tokio::io::SeekFrom;
use tokio::net::UdpSocket;

use crate::{GlobalConfig, FileSend, UdpPackage, Task, FileInfo, NotReceived};
use bincode;

pub async fn build_tunnel(
    lsocket: &Arc<UdpSocket>,
    config: &Arc<GlobalConfig>,
    remote_addr: &SocketAddr,
) -> Result<()> {
    // 制造隧道
    println!("### 发送建立隧道信息");
    let bridge_node_byte = bincode::serialize(&UdpPackage {
        cmd: 4,
        buf: bincode::serialize(&remote_addr).unwrap(),
    })
    .unwrap();
    let _ret = lsocket
        .send_to(&bridge_node_byte, &config.echo_server)
        .await?;
    // };
    Ok({})
}