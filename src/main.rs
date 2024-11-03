use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

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
                let response = "+PONG\r\n";
                if let Err(e) = stream.write_all(response.as_bytes()).await {
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
