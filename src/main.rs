extern crate getopts;
extern crate httparse;
extern crate bufstream;
extern crate time;

use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
// use std::io::Read;
use std::fs::File;
use std::thread;
use std::process;
use std::env;

use bufstream::BufStream;
use getopts::Options;
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
        path = req.path.unwrap();
        method = req.method.unwrap();
    }

    // incomplete request
    if method.to_string().is_empty() || path.to_string().is_empty() {
        let res = b"HTTP/1.0 404 Not Found\n \
                    Connection: close\n";
        bs.write(res);
        return;
    } else if method.eq("POST") {
        // only allows get requests
        let res = b"HTTP/1.0 405 Method Not Allowed\n \
                    Connection: close\n";
        bs.write(res);
        return;
    }
    
    send_response(bs.get_mut(), path, method);
}

fn send_response(stream: &mut TcpStream, path: &str, method: &str) {

    // check if the file exists
    let trimmed = path.trim_left_matches('/');
    println!("User requsted {}", trimmed);
    let mut file = match File::open(trimmed) {
        Ok(file) => { file },
        Err(e) => {
            let res = b"HTTP/1.0 404 Not Found\n \
                        Connection: close\n";
            let _ = stream.write(res);
            return;
        }
    };

    let mut contents = Vec::new();
    file.read_to_end(&mut contents);
    println!("{}", contents.len());

    let mut now = time::now();
    
    let res = format!("HTTP/1.0 200 OK\n\
                       Date: {}\n\
                       Content-Length: {}\n\
                       Connection: close\n\r\n", now.ctime(), contents.len());
    let _ = stream.write(&*res.into_bytes());

    let _ = stream.write(&*contents);
    stream.flush();
}

fn usage() {
    println!("Usage: ./erode [-dlh] -h HOST -p PORT");
    process::exit(0);
}

fn main() {
    // grab the args
    let args: Vec<_> = env::args().collect();
    let program = args[0].clone();
    // if a.len() != 3 {
    //     usage();
    // }

    let mut opts = Options::new();
    opts.optflag("d", "daemon", "Daemonize the server");
    opts.optflag("l", "log", "Turn on logging to the console");
    opts.reqopt("a", "address", "Local address to bind to", "ADDRESS");
    opts.reqopt("p", "port", "Local port to bind to", "PORT");
    opts.optflag("h", "help", "Print help");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };
    if matches.opt_present("h") {
        usage();
        return;
    }

    let address = matches.opt_str("a").unwrap();
    let port = matches.opt_str("p").unwrap();
    

    // combine the strings into the addrress
    // let addr = args[1].to_string() + ":" + &args[2];
    let addr = address.to_string() + ":" + &port;

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
