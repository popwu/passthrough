use anyhow::Ok;
use anyhow::Result;
use uuid::Uuid;
use std::env;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::path::Path;

use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tokio::sync::Mutex;
use tokio::fs::File;
use tokio::fs::metadata;

mod task_keep;
use task_keep::keepalive;

mod sub_fn;
use sub_fn::*;

mod loop_main;
use loop_main::*;

mod loop_udp;
use loop_udp::*;

mod gconfig;
use gconfig::*;

mod protocol;
use protocol::*;

#[tokio::main]
async fn main() -> Result<()> {
    // user's guidance
    // transfer get | receive file
    // transfer send ip:port filename | send file
    // transfer echoip

    let local_addr: String;
    let action: String;
    let mut remote_addr: String;
    let mut filename: String;

    remote_addr = "".to_string();
    filename = "".to_string();

    let env_length = env::args().len();
    if env_length == 1 {
        action = "get".to_string();
        local_addr = "0.0.0.0:8880".to_string();
    } else if env_length == 2 && env::args().nth(1).unwrap() == "echoip".to_string() {
        action = "echoip".to_string();
        local_addr = "0.0.0.0:8882".to_string();
    } else if env_length == 4 && env::args().nth(1).unwrap() == "send".to_string() {
        action = "send".to_string();
        local_addr = "0.0.0.0:8881".to_string();
        remote_addr = env::args().nth(2).unwrap();
        filename = env::args().nth(3).unwrap();
    } else {
        println!("user's guidance");
        println!("transfer | start a node to receive file");
        println!("transfer send ip:port filename | send file");
        println!("transfer echoip | start a echoip server");
        return Ok({});
    }

    // build socket
    let socket = UdpSocket::bind(&local_addr).await?;
    let socket = Arc::new(socket);
    println!("Listening on: {}", socket.local_addr()?);

    // build config
    let config = Arc::new(GlobalConfig {
        self_addr: Mutex::new(None),
        fp: Mutex::new(None),
        sleep_time: 60000, //180000,
        filename: "".to_string(),
        action: action.clone(),
        echo_server: "127.0.0.1:8882".to_string(),
        // tasks: Mutex::new(HashMap::new()),
    });

    // task var
    let (tx, rx): (mpsc::Sender<ReceiveBuf>, mpsc::Receiver<ReceiveBuf>) = mpsc::channel(1024);
    let mut tasks:HashMap<Uuid, Task> = HashMap::new();

    // sub task
    let lconfig = Arc::clone(&config);
    let lsocket = socket.clone();
    if action == "get".to_string() {
        tokio::spawn(async move { keepalive(&lsocket, &lconfig).await });
    } else if action == "send".to_string() {
        // 添加一个发送文件的任务
        let file_id = Uuid::new_v4();
        let remote_addr: SocketAddr = remote_addr.parse().unwrap();
        let fp = File::open(&filename).await?;
        let file_size = metadata(&filename).await.unwrap().len();
        println!("### 建立发送任务 id:{:?} size:{:?}", file_id, file_size);

        let b_file_name = Path::new(&filename).file_name().unwrap();
        let sft = SendFileTask{
            filename: b_file_name.to_str().unwrap().as_bytes().to_vec(),
            size: file_size,
            remote_addr: remote_addr.clone(),
            fp: fp,
        };
        let _ret = tasks.insert(file_id, Task::SendFile(sft));

        // 开通隧道
        tokio::spawn(async move { build_tunnel(&lsocket, &lconfig, &remote_addr).await });
    }

    let rp_socket = socket.clone();
    let rp_config = Arc::clone(&config);
    tokio::spawn(async move { main_loop(rp_socket, rp_config, rx, tasks).await });

    // main task
    let _ret = loop_udp(socket, config, tx).await?;
    Ok(())
}
