use crate::resp::{bytes_to_resp, RESP};
use crate::server::process_request;
use crate::storage::Storage;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
mod resp;
mod resp_result;
mod server;
mod storage;
mod storage_result;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;

    let storage = Arc::new(Mutex::new(Storage::new()));

    let mut interval_timer = tokio::time::interval(Duration::from_millis(10));
    loop {
        tokio::select! {
                connection = listener.accept() => {
                    match connection {
                        Ok((stream, _)) => {
                            tokio::spawn(handle_connection(stream, storage.clone()));
                        }
                        Err(e) => {
                            println!("Error: {}", e);
                            continue;
                        }
                    }
                }

                _ = interval_timer.tick() => {
                    tokio::spawn(expire_keys(storage.clone()));
                }
        }
    }
}

async fn handle_connection(mut stream: TcpStream, storage: Arc<Mutex<Storage>>) {
    let mut buff = [0; 512];

    loop {
        match stream.read(&mut buff).await {
            Ok(size) if size != 0 => {
                let mut index: usize = 0;
                let request = match bytes_to_resp(&buff[..size].to_vec(), &mut index) {
                    Ok(v) => v,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        return;
                    }
                };
                let response = match process_request(request, storage.clone()) {
                    Ok(v) => v,
                    Err(_) => {
                        eprintln!("Error processing request");
                        return;
                    }
                };
                if let Err(e) = stream.write_all(response.to_string().as_bytes()).await {
                    eprintln!("Error writing to socket: {}", e);
                }
            }
            Ok(_) | Err(_) => {
                match stream.peer_addr() {
                    Ok(addr) => {
                        println!("{} Connection closed", addr);
                    }
                    Err(e) => {
                        println!("Connection closed: {}", e);
                    }
                }
                return;
            }
            Err(e) => {
                println!("Error: {}", e);
                return;
            }
        }
    }
}

async fn expire_keys(storage: Arc<Mutex<Storage>>) {
    let mut guard = match storage.lock() {
        Ok(guard) => guard,
        Err(e) => {
            eprintln!("Error locking storage: {}", e);
            return;
        }
    };
    guard.expire_keys();
}
