use anyhow::{Ok, Result};
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::fs::{metadata, File};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt, SeekFrom};
use tokio::net::UdpSocket;
use tokio::sync::mpsc;

use crate::gconfig::ReceiveBuf;
use crate::{FileSend, GlobalConfig, UdpPackage, send_file_step2};

pub async fn resolve_package(socket: Arc<UdpSocket>, config: Arc<GlobalConfig>, mut rx: mpsc::Receiver<ReceiveBuf>) -> Result<()> {
    while let Some(message) = rx.recv().await {
        // println!("GOT = {}", message);
        let ReceiveBuf {
            buf,
            from_addr,
        } = message;

        // println!("in resolve_package {:?}",buf);
        let pack: UdpPackage = bincode::deserialize(&buf).unwrap();

        // ### echoip server
        if pack.cmd == 3 {
            // let b = bincode::serialize(from_addr).unwrap();
            let up = UdpPackage {
                cmd: 1,
                buf: bincode::serialize(&from_addr).unwrap(),
            };
            let up_byte = bincode::serialize(&up).unwrap();
            // let pack: UdpPackage = bincode::deserialize(&bb).unwrap();
            let _ret = socket.send_to(&up_byte, &from_addr).await?;
            println!("from address: {}  say: {:?}", from_addr, pack.buf);
        }
        if pack.cmd == 4 {
            if config.action == "send".to_string() {
                let _ret = send_file_step2(&socket, &config, &from_addr).await?;
            } else {
                let addr: SocketAddr = bincode::deserialize(&pack.buf).unwrap();
                let bridge_node_byte = bincode::serialize(&UdpPackage {
                    cmd: 4,
                    buf: bincode::serialize(&from_addr).unwrap(),
                })
                .unwrap();
                let _ret = socket.send_to(&bridge_node_byte, addr).await?;
                println!("from address: {}  bridge to: {:?}", from_addr, addr);
            }
        }

        // server -> node ip address
        if pack.cmd == 1 {
            let addr: SocketAddr = bincode::deserialize(&pack.buf).unwrap();

            // let mut config = config.lock().await;
            if Some(addr) != *config.self_addr.lock().await {
                println!("new address: {:?}", addr);
                println!("sleep: {:?}", config.sleep_time);
                *config.self_addr.lock().await = Some(addr);
            }
        }

        //
        if pack.cmd == 2 {
            let file_send: FileSend = bincode::deserialize(&pack.buf).unwrap();

            // let mut config = config.lock().await;
            let mut fp = config.fp.lock().await;
            if fp.is_none() {
                println!("create file: {:?}", &config.filename);
                let a = File::create(&config.filename).await?;
                *fp = Some(a);
            }
            // if let Some(mut fp) = &*fp {
            println!("cmd 2 index:{}", file_send.index);
            let _ret = fp
                .as_mut()
                .unwrap()
                .seek(SeekFrom::Start(file_send.index * 1024))
                .await?;
            let _ret = fp.as_mut().unwrap().write(&file_send.buf).await?;
            let _ret = fp.as_mut().unwrap().flush().await?;
            // }
        }
    }
    Ok(())
}
