use anyhow::{Ok, Result};
use uuid::Uuid;
use std::net::SocketAddr;
use std::sync::Arc;
use std::collections::HashMap;

use tokio::fs::File;
use tokio::io::{AsyncSeekExt, AsyncWriteExt, SeekFrom};
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use crate::gconfig::ReceiveBuf;
use crate::{Task, ReceiveFileTask};

mod sub_fn;
use sub_fn::*;

mod protocol;
use protocol::{FileInfo, FileSend, UdpPackage, NotReceived};

pub async fn build_tunnel(
    lsocket: Arc<UdpSocket>,
    echo_server: SocketAddr,
    remote_addr: SocketAddr,
) -> Result<()> {
    // 制造隧道
    println!("### 发送建立隧道信息");
    let bridge_node_byte = bincode::serialize(&UdpPackage {
        cmd: 4,
        buf: bincode::serialize(&remote_addr).unwrap(),
    })
    .unwrap();
    let _ret = lsocket
        .send_to(&bridge_node_byte, echo_server)
        .await?;
    // };
    Ok({})
}

pub async fn keepalive(lsocket: Arc<UdpSocket>, echo_server: SocketAddr, sleep_time: u64) -> Result<()> {
    let who_am_i_byte = bincode::serialize(&UdpPackage {
        cmd: 3,
        buf: vec![0],
    })
    .unwrap();    
    loop {
        let _ret = lsocket.send_to(&who_am_i_byte, echo_server).await?;
        let _ret = sleep(Duration::from_millis(sleep_time)).await;
    }
}

// 1 server -> node 接收方: 获得自己的ip
// 2 node -> node   发送方: 文件切片
// 3 node -> server 服务器: 返回ip信息
// 4 node -> server 发送方: 建立隧道
// 5 node -> node   发送方: 文件发送结束
// 6 node -> node   发送方: 发送文件信息
// 7 node -> node   接收方: 我要这个文件
// 8 node -> node   接收方: 还有些我没有收到

pub async fn main_loop(
    socket: Arc<UdpSocket>, 
    // config: Arc<GlobalConfig>, 
    mut rx: mpsc::Receiver<ReceiveBuf>,
    mut tasks: HashMap<Uuid, Task>,
    action: String,
) -> Result<()> {
    let mut self_addr: Option<SocketAddr> = None;

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
            if action == "send".to_string() {
                let _ret = send_file_for_remote_addr(&socket, &from_addr, &tasks).await?;
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
            if Some(addr) != self_addr {
                println!("new address: {:?}", addr);
                self_addr = Some(addr);
            }
        }

        // node -> node
        if pack.cmd == 2 {
            let file_send: FileSend = bincode::deserialize(&pack.buf).unwrap();
            // println!("### 收到切片 {:?}", file_send.index);
            if let Task::ReceiveFile(task) = tasks.get_mut(&file_send.id).unwrap() {
                // let _ret = task.received; //.(file_send.index as usize);
                task.received.remove(task.received.iter().position(|x| *x == file_send.index).expect("not found"));
                // fp = task.fp;
            
            // let mut config = config.lock().await;
            // let mut fp = config.fp.lock().await;
            // let fp = 
            // if fp.is_none() {
            //     println!("create file: {:?}", &config.filename);
            //     let a = File::create(&config.filename).await?;
            //     *fp = Some(a);
            // }
            // if let Some(mut fp) = &*fp {
            // println!("cmd 2 index:{}", file_send.index);
            let _ret = task.fp
                .seek(SeekFrom::Start(file_send.index * 1024))
                .await?;
            let _ret = task.fp.write(&file_send.buf).await?;
            // let _ret = fp.as_mut().unwrap().flush().await?;
        }
            
        }
        if pack.cmd == 6 {
            let file_info: FileInfo = bincode::deserialize(&pack.buf).unwrap();
            
            // 添加一个接收文件的任务
            
            let file_id = file_info.id;
            let l = if file_info.size % 1024 == 0 {
                file_info.size / 1024
            } else {
                file_info.size / 1024 + 1
            };
            let received: Vec<u64> = (0..l).collect();
            let filename = String::from_utf8(file_info.filename).unwrap();
            let fp = File::create(&filename).await?;

            println!("### 收到一个文件的请求建立接收任务 {:?} index:{:?} len:{:?}", file_info.id, l, received.len());

            let sft = ReceiveFileTask{
                // filename: file_info.filename.clone(),
                from_addr: from_addr,
                received: received,
                fp: fp,
            };
            
            let _ret = tasks.insert(file_id, Task::ReceiveFile(sft));

            // 返回一个许可
            println!("### 返回我要这个文件的许可 {:?}", file_id);
            let file_id_byte = bincode::serialize(&UdpPackage {
                cmd: 7,
                buf: bincode::serialize(&file_id).unwrap(),
            })
            .unwrap();
            let _ret = socket.send_to(&file_id_byte, from_addr).await?;
        }
        if pack.cmd == 7 {
            let file_id: Uuid = bincode::deserialize(&pack.buf).unwrap();
            let _ret = send_file(&socket, &file_id, &mut tasks).await?;
        }
        if pack.cmd == 5 {
            let file_id: Uuid = bincode::deserialize(&pack.buf).unwrap();
            let received;
            // let t = config.tasks.lock().await.get(&file_id).unwrap();
            if let Task::ReceiveFile(task) = tasks.get(&file_id).unwrap() {
                received = task.received.clone();
            } else {
                received = Vec::new();
            }
            if received.len() != 0 {
                // 发送继续
                let not_received = NotReceived{
                    id: file_id,
                    not_received: received,
                };
                let not_received_byte = bincode::serialize(&UdpPackage {
                    cmd: 8,
                    buf: bincode::serialize(&not_received).unwrap(),
                })
                .unwrap();
                let _ret = socket.send_to(&not_received_byte, from_addr).await?;
            } else {
                // 结束
                if let Task::ReceiveFile(task) = tasks.get_mut(&file_id).unwrap() {
                    let _ret = task.fp.flush().await.unwrap();
                    let _ret = task.fp.try_clone().await.unwrap();
                }
            }
        }
        if pack.cmd == 8 {
            let not_received: NotReceived = bincode::deserialize(&pack.buf).unwrap();
            let _ret = send_not_received(&socket, &not_received, &mut tasks).await.unwrap();
        }
    }
    Ok(())
}
