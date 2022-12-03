use anyhow::{Result, Ok};
use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::fs::{metadata, File};
use tokio::io::AsyncReadExt;
use tokio::io::AsyncSeekExt;
use tokio::io::SeekFrom;
use crate::{UdpPackage, FileSend};
use bincode;

pub async fn send_file(filename: &String, socket:Arc<UdpSocket>, remote_addr: &String) -> Result<()>  {
    let file_size = metadata(filename).await.unwrap().len();
    let l = if file_size%1024 == 0 { file_size/1024} else { file_size/1024+1};

    let mut f = File::open(filename).await?;
    
    for i in 0..l {
        println!("{}",i);
        let mut buf = vec![0;1024];

        let _ret = f.seek(SeekFrom::Start(i*1024)).await?;
        let n = f.read(&mut buf).await?;

        let file_send = FileSend{
            // filename : filename.as_bytes().to_vec(),
            size: file_size,
            index: i,
            buf: buf[..n].to_vec(),

        };
        let file_send_byte = bincode::serialize(&file_send).unwrap();

        let send = UdpPackage{
            cmd:2,
            buf:file_send_byte,
        };
        let send_byte = bincode::serialize(&send).unwrap();
        let _ret = socket.send_to(&send_byte, &remote_addr).await?;
    }
    println!("send is ok.");
    Ok({})
}