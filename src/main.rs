extern crate httparse;
extern crate bufstream;

use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
// use std::io::Read;
use std::thread;
use std::process;
use std::env;

use bufstream::BufStream;
// use httparse::Request;


fn handle_client(stream: TcpStream) {
    let addr = stream.peer_addr().unwrap();
    let mut bs = BufStream::new(stream);
    println!("Connection from {}", addr);

    let mut buf = [0; 1024];
    let mut path = "";
    let mut method = "";
    // stream.read(&mut buf);
    let _ = bs.read(&mut buf);
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);
    let res = req.parse(&mut buf).unwrap();
    if res.is_complete() {
        if req.path.is_some() && req.method.is_some() {
            match req.path {
                Some(p) => {
                    path = p;
                },
                None => {
                    return;
                }
            }
            match req.method {
                Some(m) => {
                    method = m;
                },
                None => {
                    return;
                }
            }
        }
    }

    if method.to_string().is_empty() || path.to_string().is_empty() {
        return;
    }
    
    send_response(bs.get_mut(), path, method);
}

fn send_response(stream: &mut TcpStream, path: &str, method: &str) {
    let res = b"HTTP/1.0 200 OK\n \
                Connection: close\n";
    let _ = stream.write(res);
}

fn usage() {
    println!("Usage: ./erode [host] [port]");
    process::exit(0);
}

fn main() {
    // grab the args
    let a: Vec<_> = env::args().collect();

    if a.len() != 3 {
        usage();
    }

    // combine the strings into the addrress
    let addr = a[1].to_string() + ":" + &a[2];

    // TODO - Fix Panics from invalid addresses
    let listener = TcpListener::bind(&*addr).unwrap();

    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move|| {
                    handle_client(stream)
                });
            }
            Err(e) => {
                println!("Connection failed: {}", e);
            }
        }
    }
    // close the socket server
    drop(listener);
}
