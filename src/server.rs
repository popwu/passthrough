// use tokio::net::UdpSocket;
use anyhow::{Ok, Result};
use bincode;
use tokio::net::UdpSocket;

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom, AsyncWriteExt};

use crate::GlobalConfig;
use crate::{UdpPackage, FileSend};


pub struct Server {
    pub socket: Arc<tokio::net::UdpSocket>,
    pub buf: Vec<u8>,
    pub to_send: Option<(usize, SocketAddr)>,
    pub config: Arc<Mutex<GlobalConfig>>,
}

async fn resolve_package(buf: Vec<u8>, config: &Arc<Mutex<GlobalConfig>>, from_addr: &SocketAddr, socket: Arc<UdpSocket>) -> Result<()> {
    // println!("in resolve_package {:?}",buf);
    let mut pack: UdpPackage = bincode::deserialize(&buf).unwrap();

    // ### echoip server
    if pack.cmd == 3 {
        // let b = bincode::serialize(from_addr).unwrap();
        let up = UdpPackage{
            cmd: 1,
            buf: bincode::serialize(from_addr).unwrap()
        };
        let up_byte = bincode::serialize(&up).unwrap();
        // let pack: UdpPackage = bincode::deserialize(&bb).unwrap();
        let _ret = socket.send_to(&up_byte, &from_addr).await?;
        println!("from address: {}  say: {:?}", from_addr, pack.buf);
    }
    
    // server -> node ip address
    if pack.cmd == 1 {
        let addr: SocketAddr = bincode::deserialize(&pack.buf).unwrap();
        {
            let mut config = config.lock().await;
            if Some(addr) != config.self_addr {
                println!("new address: {:?}", addr);
                println!("sleep: {:?}", config.sleep_time);
                config.self_addr = Some(addr);
            }
        }
    }

    // 
    if pack.cmd == 2 {
        let file_send: FileSend = bincode::deserialize(&pack.buf).unwrap();
        
        let mut config = config.lock().await;
        if config.fp.is_none() {
            config.fp = Some(File::create(&config.filename).await?);
        }
        if let Some(fp) = &mut config.fp {
            println!("cmd 2 index:{}", file_send.index);
            let _ret = fp.seek(SeekFrom::Start(file_send.index*1024)).await?;
            let _ret = fp.write(&file_send.buf).await?;
            let _ret = fp.flush().await?;
        }

    }
    Ok(())
}

impl Server {
    pub async fn run(self) -> Result<()> {
        let Server {
            socket,
            mut buf,
            to_send,
            config,
        } = self;

        loop {
            let (size, from_addr) = socket.recv_from(&mut buf).await?;
            if size != 0 {
                let mut new_buf: Vec<u8> = vec![0; size];
                let _a = new_buf.clone_from_slice(&buf[..size]);
                let _a = resolve_package(new_buf, &config, &from_addr, &socket).await?;
            }
            // First we check to see if there's a message we need to echo back.
            // If so then we try to send it back to the original source, waiting
            // until it's writable and we're able to do so.
            // if let Some((size, peer)) = to_send {
            // let mut nsocket = socket.blocking_lock();
            // let amt = socket.send_to(&buf[..size], &peer).await?;

            // println!("Echoed {}/{} bytes to {}", amt, size, peer);
            // }

            // If we're here then `to_send` is `None`, so we take a look for the
            // next message we're going to echo back.
            // let mut nsocket = socket.lock().await;
        }
    }
}
