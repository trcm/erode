// extern crate httparse;

use std::net::{TcpListener, TcpStream};
use std::io::Read;
use std::thread;

// use httparse::Request;

fn handle_client(stream: TcpStream) {
    println!("connection");
    let _ = stream.read(ref &mut [0; 128]);

    // let mut headers = [httparse::EMPTY_HEADER; 16];
    // let mut req = httparse::Request::new(&mut headers);
    // let res = req.parse(&buff).unwrap();
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:1234").unwrap();

    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move|| {
                    handle_client(stream)
                });
            }
            Err(e) => {
                println!("connection failed");
            }
        }
    }
    // close the socket server
    drop(listener);
}
