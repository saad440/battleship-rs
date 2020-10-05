use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{Read, Error};

fn connection_handler(stream: TcpStream) -> Result<(), Error> {
    println!("New client {}", stream.peer_addr()?);
    Ok(())
}

fn main() -> Result<(), Error> {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                connection_handler(stream)?;
            }
            Err(e) => { println!("Error: {}", e); }
        }
    }

    Ok(())
}