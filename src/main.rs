use bincode;
use std::error::Error;

use std::sync::Arc;
use std::{env, io};
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::time::error::Elapsed;
use std::net::SocketAddr;

mod keep;
use keep::keepalive;

mod server;
use server::Server;

mod gconfig;
use gconfig::GlobalConfig;

mod sendfile;
use sendfile::send_file;

mod protocol;
use protocol::*;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // transfer get filename | receive file

    // transfer ip:port filename | send file

    let local_addr: String;
    let action: String;
    let remote_addr: String;

    let args1 = env::args().nth(1).unwrap();
    if args1 == "get".to_string() {
        action = "get".to_string();
        local_addr = "0.0.0.0:8880".to_string();
        remote_addr = "".to_string();

        // let filename: String = env::args().nth(2).unwrap();

        
    } else {
        action = "send".to_string();
        local_addr = "0.0.0.0:8881".to_string();
        remote_addr = args1.to_string();

        
        // let socket = UdpSocket::bind(&local_addr).await?;

        // send_file(filename, socket, remote_addr);

    }
    let filename: String = env::args().nth(2).unwrap();
    let _socket = UdpSocket::bind(&local_addr).await?;
    println!("Listening on: {}", _socket.local_addr()?);

    let socket = Arc::new(_socket);
    let lsocket = socket.clone();

    let config = Arc::new(Mutex::new( GlobalConfig{
        self_addr: None,
        sleep_time: 60000, //180000,
        filename: filename.clone(),
        fp: None,
    }));
    let lconfig = Arc::clone(&config);

    if action == "get".to_string() {
        tokio::spawn(async move { keepalive(lsocket, lconfig).await });
    }
    else {
        tokio::spawn(async move { send_file(&filename, lsocket, &remote_addr).await });
    }
    
    let server = Server {
        socket,
        buf: vec![0; 2048],
        to_send: None,
        config: config,
    };

    // This starts the server task.
    server.run().await?;
   
    

    Ok(())
}
