use std::net::TcpStream;
use std::io::{Read, Write, Error};
use std::str::from_utf8;

fn main() -> Result<(), Error> {
    let conn = TcpStream::connect("localhost:8080");
    match conn {
        Ok(mut stream) => { println!("Successfully Connected to {}", stream.peer_addr()?); }
        Err(e) => { println!("Error: {}", e); }

    }

    Ok(())
}

/* 
use rand::Rng;
use libbattleships::{Board, Position};

 fn main() {
     let mut board = Board::new();
     board.setup();
     board.display_board();
     println!("= = = = = = = = =");
     for _ in 1..100 {
         let random_x: i32 = rand::thread_rng().gen_range(0,9);
         let random_y: i32 = rand::thread_rng().gen_range(0,9);
         let random_pos = Position::new(random_x, random_y);
         board.hit_cell(random_pos);
     }
     board.display_board();
 }
 */
 