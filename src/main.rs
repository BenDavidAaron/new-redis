use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                handle_connection(&mut stream);
            }
            Err(e) => {
                println!("Connection failed: {}", e);
            }
        }
    }
}

fn handle_connection(stream: &mut TcpStream) {
    let mut buff = [0; 512];

    loop {
        match stream.read(&mut buff) {
            Ok(size) if size != 0 => {
                let response = "+PONG\r\n";
                stream.write(response.as_bytes()).unwrap();
                stream.flush().unwrap();
            }
            Ok(_) | Err(_) => {
                println!("Connection closed");
                break;
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
}
