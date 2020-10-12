use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::{BufRead, Write, BufReader, BufWriter, Error};
use std::thread;
use libbattleship::{Board, CommandResult, command_handler, command_parser};


fn connection_handler(stream: TcpStream) -> Result<(), Error> {
    println!("New client {}", stream.peer_addr()?);
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);
    let mut board: Option<Board> = None;

    loop {
        let mut response = String::new();
        let bytes_read = reader.read_line(&mut response)?;
        if bytes_read == 0 {
            println!("Client {} disconnected.", stream.peer_addr()?);
            return Ok(())
        }
        let resp = response.trim();
        println!("{}: {}", stream.peer_addr()?, resp);

        let cmd = command_parser(resp);
        println!("Received command: {:?}", cmd);
        let result = command_handler(&mut board, cmd);

        match result {
            CommandResult::Success(msg) => {
                writer.write(String::as_bytes(&format!("{}\n",msg)))?;
                writer.flush()?;
                println!("SUCCESS: {}", msg);
                },
            CommandResult::Failure(msg) => {
                writer.write(String::as_bytes(&format!("{}\n",msg)))?;
                writer.flush()?;
                println!("FAILURE: {}", msg);
                },
            CommandResult::Message(msg) => {
                writer.write(String::as_bytes(&format!("{}\n",msg)))?;
                writer.flush()?;
                println!("{}", msg);
                },
            CommandResult::Some(b) => {
                writer.write(b"Starting new game.\n")?;
                writer.flush()?;
                board = Some(b); println!("Creating new board");
                },
            CommandResult::None => {
                writer.write(b"Nothing to do\n")?;
                writer.flush()?;
                println!("Nothing to do");
                },
            CommandResult::GameComplete(score) => {
                writer.write(b"Game successcully completed.\n")?;
                writer.flush()?;
                println!("Game successcully completed. Score {}",score);
                stream.shutdown(Shutdown::Both).expect("shutdown call failed");
                return Ok(());
            }
            CommandResult::Quit => {
                println!("Client quit. Closing connection.");
                stream.shutdown(Shutdown::Both).expect("shutdown call failed");
                return Ok(());
            },

        }

    }
    
}


fn main() -> Result<(), Error> {
    let listener = TcpListener::bind("0.0.0.0:8888").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || -> Result<(), Error> {
                    connection_handler(stream)?;
                    Ok(())
            });
            }
            Err(e) => { println!("Error: {}", e); }
        }
    }

    Ok(())
}
