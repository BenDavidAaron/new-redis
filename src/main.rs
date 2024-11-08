use crate::resp::{bytes_to_resp, RESP};
use crate::server::process_request;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};
mod resp;
mod resp_result;
mod server;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                tokio::spawn(handle_connection(stream));
            }
            Err(e) => {
                println!("Error: {}", e);
                continue;
            }
        }
    }
}

async fn handle_connection(mut stream: TcpStream) {
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
                let response = match process_request(request) {
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
