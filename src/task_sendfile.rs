use anyhow::{Ok, Result};
use uuid::Uuid;
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::fs::{metadata, File};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use tokio::io::SeekFrom;
use tokio::net::UdpSocket;

// use crate::GlobalConfig;
// use crate::gconfig::Task;
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

pub async fn send_file_for_remote_addr(
    socket: &Arc<UdpSocket>,
    config: &Arc<GlobalConfig>,
    remote_addr: &SocketAddr,
) -> Result<()> {
    println!("### {:?} 回应了隧道，隧道已经建立", remote_addr);
    // let tasks = &*config.tasks.lock().await;
    let mut  file_ids: Vec<Uuid> = Vec::new();
    for (file_id, vtask) in  &*config.tasks.lock().await{
        if let Task::SendFile(task) = vtask {
            if task.remote_addr == *remote_addr {
                file_ids.insert(0, file_id.clone());
            }
        }
    };
    for file_id in file_ids
    {
        let _ret = send_file_info(socket, config, &file_id).await?;
    };
    Ok({})

}

pub async fn send_file_info(
    socket: &Arc<UdpSocket>,
    config: &Arc<GlobalConfig>,
    file_id: &Uuid,
) -> Result<()> {
    if let Task::SendFile(task) = config.tasks.lock().await.get(&file_id).unwrap() {
        // let filename = String::from_utf8(task.filename.clone()).unwrap();
        let remote_addr = task.remote_addr;
        let file_size = task.size;
        let l = if file_size % 1024 == 0 {
            file_size / 1024
        } else {
            file_size / 1024 + 1
        };

        println!("### 发送文件信息{:?}", file_id);

        // 发送概述
        let fi = FileInfo {
            filename: task.filename.clone(),
            id: *file_id,
            size: file_size,
        };
        let file_info_byte = bincode::serialize(&fi).unwrap();
        let send = UdpPackage {
            cmd: 6,
            buf: file_info_byte,
        };
        let send_byte = bincode::serialize(&send).unwrap();
        let _ret = socket.send_to(&send_byte, remote_addr).await?;
    }
    Ok({})

}

pub async fn send_file(
    socket: &Arc<UdpSocket>,
    config: &Arc<GlobalConfig>,
    file_id: &Uuid,
) -> Result<()> {
    // 一次性发送所有内容
    println!("### 一次性发送所有内容 {:?}", file_id);
    if let Task::SendFile(task) = config.tasks.lock().await.get_mut(&file_id).unwrap() {
        // let filename = String::from_utf8(task.filename.clone()).unwrap();
        let remote_addr = task.remote_addr;
        let file_size = task.size;
        let l = if file_size % 1024 == 0 {
            file_size / 1024
        } else {
            file_size / 1024 + 1
        };
        println!("### 文件 size: {:?}", file_size);
        
        // 发送文件
        for i in 0..l {
            // println!("##### 发送切片 {}", i);
            let mut buf = vec![0; 1024];

            let _ret = task.fp.seek(SeekFrom::Start(i * 1024)).await?;
            let n = task.fp.read(&mut buf).await?;

            let file_send = FileSend {
                id: *file_id,
                index: i,
                buf: buf[..n].to_vec(),
            };
            let file_send_byte = bincode::serialize(&file_send).unwrap();

            let send = UdpPackage {
                cmd: 2,
                buf: file_send_byte,
            };
            let send_byte = bincode::serialize(&send).unwrap();
            let _ret = socket.send_to(&send_byte, remote_addr).await?;
        }
        // println!("send is ok.");

        // 发送完毕信息
        println!("### 发送完毕 {:?}", file_id);
        let file_id_byte = bincode::serialize(&UdpPackage {
            cmd: 5,
            buf: bincode::serialize(&file_id).unwrap(),
        })
        .unwrap();
        let _ret = socket.send_to(&file_id_byte, remote_addr).await?;

        
    }
    Ok({})
}

pub async fn send_not_received(
    socket: &Arc<UdpSocket>,
    config: &Arc<GlobalConfig>,
    not_received: &NotReceived,
) -> Result<()> {
    // let (id, not_received} = not_received;
    // 一次性发送所有内容
    if let Task::SendFile(task) = config.tasks.lock().await.get_mut(&not_received.id).unwrap() {
        // let f = task.fp;
        let remote_addr = task.remote_addr;
        for i in &not_received.not_received {
            println!("{}", i);
            let mut buf = vec![0; 1024];

            let _ret = task.fp.seek(SeekFrom::Start(i * 1024)).await?;
            let n = task.fp.read(&mut buf).await?;

            let file_send = FileSend {
                id: not_received.id,
                index: *i,
                buf: buf[..n].to_vec(),
            };
            let file_send_byte = bincode::serialize(&file_send).unwrap();

            let send = UdpPackage {
                cmd: 2,
                buf: file_send_byte,
            };
            let send_byte = bincode::serialize(&send).unwrap();
            let _ret = socket.send_to(&send_byte, remote_addr).await?;
        }
        // println!("send is ok.");

        // 发送完毕信息
        println!("### 补发完毕 {:?}", not_received.id);
        let file_id_byte = bincode::serialize(&UdpPackage {
            cmd: 5,
            buf: bincode::serialize(&not_received.id).unwrap(),
        })
        .unwrap();
        let _ret = socket.send_to(&file_id_byte, remote_addr).await?;
    }
    Ok({})
}