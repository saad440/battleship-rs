use std::net::TcpStream;
use std::io::{self, BufRead, Write, BufReader, BufWriter, Error};


fn main() -> Result<(), Error> {
    let stream = TcpStream::connect("localhost:8888")?;
    println!("Successfully Connected to {}", stream.peer_addr()?);
    let mut reader = BufReader::new(&stream);
    let mut writer = BufWriter::new(&stream);

    /*
    // Automated play to test server-client interaction.
    writer.write(b"STARTGAME\n")?;
    writer.flush()?;
    for x in 1..=9 {
        for y in 1..=9 {
            let cmd = format!("CELL:[{},{}]\n", x,y);
            writer.write(&(cmd.clone().into_bytes()))?;
            writer.flush()?;
            let mut buff = String::new();
            reader.read_line(&mut buff)?;
        
            println!("Server: {}", buff.trim());
        }
    }
    */

    loop {
        print!("> ");
        io::stdout().flush()?;
        let mut inpt = String::new();
        io::stdin().read_line(&mut inpt).expect("reading from stdin failed");
        let msg = inpt.trim().to_string();
        if msg == ":q" {break}
        let mut m = msg.clone().into_bytes();
        m.push(0xA);
        writer.write(&m)?;
        writer.flush()?;

        if msg == "QUIT" {break}

        let mut buff = String::new();
        reader.read_line(&mut buff)?;
        
        println!("Server: {}", buff.trim());
    }

    Ok(())
}
