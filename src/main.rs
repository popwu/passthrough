use std::error::Error;
use std::sync::Arc;
use std::env;

use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::sync::mpsc;

mod task_keep;
use task_keep::keepalive;

mod task_sendfile;
use task_sendfile::*;

mod task_processmess;
use task_processmess::*;

mod task_server;
use task_server::*;

mod gconfig;
use gconfig::*;

mod protocol;
use protocol::*;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // user's guidance
    // transfer get filename | receive file
    // transfer ip:port filename | send file
    // transfer echoip

    let local_addr: String;
    let action: String;
    let remote_addr: String;
    let filename: String;

    let args1 = env::args().nth(1).unwrap();
    if args1 == "get".to_string() {
        action = "get".to_string();
        local_addr = "0.0.0.0:8880".to_string();
        remote_addr = "".to_string();
        filename = env::args().nth(2).unwrap();
    } else if args1 == "echoip".to_string() {
        // echoip server 模式
        action = "echoip".to_string();
        local_addr = "0.0.0.0:8882".to_string();
        remote_addr = "".to_string();
        filename = "".to_string();
    } else {
        action = "send".to_string();
        local_addr = "0.0.0.0:8881".to_string();
        remote_addr = args1.to_string();
        filename = env::args().nth(2).unwrap();
    }
    
    // build socket
    let socket = UdpSocket::bind(&local_addr).await?;
    let socket = Arc::new(socket);
    println!("Listening on: {}", socket.local_addr()?);
    

    // build config
    let (tx, rx):(mpsc::Sender<ReceiveBuf>, mpsc::Receiver<ReceiveBuf>) = mpsc::channel(1024);
    let config = Arc::new( GlobalConfig{
        self_addr: Mutex::new(None),
        fp: Mutex::new(None),
        sleep_time: 60000, //180000,
        filename: filename.clone(),
        action: action.clone(),
        echo_server: "127.0.0.1:8882".to_string(),
    });

    // sub task
    let lconfig = Arc::clone(&config);
    let lsocket = socket.clone();
    if action == "get".to_string() {
        tokio::spawn(async move { keepalive(&lsocket, &lconfig).await });
    }
    else if action == "send".to_string() {
        tokio::spawn(async move { send_file(&lsocket, &lconfig, &remote_addr).await });
    }

    let rp_socket = socket.clone();
    let rp_config = Arc::clone(&config);
    tokio::spawn(async move { resolve_package(rp_socket, rp_config, rx).await });
    

    // main task
    let _ret = server_run(socket, config, tx).await?;
    Ok(())
}
