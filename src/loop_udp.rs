// use tokio::net::UdpSocket;
use anyhow::Result;
use tokio::net::UdpSocket;

use std::sync::Arc;
use tokio::sync::mpsc;

use crate::ReceiveBuf;

// pub struct Server {
//     pub socket: Arc<tokio::net::UdpSocket>,
//     pub buf: Vec<u8>,
//     pub to_send: Option<(usize, SocketAddr)>,
//     pub config: Arc<GlobalConfig>,
// }

// impl Server {
pub async fn loop_udp(
    socket: Arc<UdpSocket>,
    tx: mpsc::Sender<ReceiveBuf>,
) -> Result<()> {

    let mut buf = vec![0; 2048];
    loop {
        let (size, from_addr) = socket.recv_from(&mut buf).await?;
        if size != 0 {
            let mut new_buf: Vec<u8> = vec![0; size];
            let _a = new_buf.clone_from_slice(&buf[..size]);
            // let _a = resolve_package(new_buf, &config, &from_addr, &socket).await?;
            let rbuf = ReceiveBuf {
                buf: new_buf,
                from_addr: from_addr.clone(),
            };
            let _ret = tx.send(rbuf).await?;
        }
    }
}
// }
