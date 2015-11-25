extern crate getopts;
extern crate httparse;
extern crate bufstream;
extern crate time;

use std::net::{TcpListener, TcpStream};
use std::io::{Read, Write};
use std::fs::File;
use std::thread;
use std::process;
use std::env;

use bufstream::BufStream;
use getopts::Options;

/// Erode
///
/// Erode is a Toy http server I wrote to help me learn Rust.
/// It is based off of a HTTP Server I have previously written for a university assignment.
///
/// Erode accepts the following arguments:
///  Required -
///  -a: address to serve the server on 
///  -p: port to listen on 
///  Options -
///  -d: daemonize the server
///  -l: toggle logging to the command line
///  -h: prints the help text

/// handle_client
/// Handling an incomming connection to the server
///
/// Arguments:
/// stream - Incoming TcpStream
fn handle_client(stream: TcpStream, dir: &String) {

    let addr = stream.peer_addr().unwrap();
    println!("Connection from {}", addr);

    // setup aa new bufferedsteram for easier reading (buffered streams allow you to read by line)
    let mut bs = BufStream::new(stream);

    let mut buf = [0; 1024];
    let mut path = "";
    let mut method = "";

    // read the request from the stream
    let _ = bs.read(&mut buf);
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let mut req = httparse::Request::new(&mut headers);

    // parse the request
    let res = req.parse(&mut buf).unwrap();

    if res.is_complete() {
        path = req.path.unwrap();
        method = req.method.unwrap();
    }

    // incomplete request
    if method.to_string().is_empty() || path.to_string().is_empty() {
        let res = b"HTTP/1.0 404 Not Found\n \
                    Connection: close\n";
        let _ = bs.write(res);
        return;
    } else if !method.eq("GET") {
        // only allows get requests
        let res = b"HTTP/1.0 405 Method Not Allowed\n \
                    Connection: close\n";
        let _ = bs.write(res);
        return;
    }

    send_response(bs.get_mut(), path, dir);
}

/// send_response:
/// will resolve the client request and response accordingly
///
/// Arguments
/// stream - mutable reference to the client TcpStream
/// path   - path to the file the user wants
fn send_response(stream: &mut TcpStream, path: &str, dir: &String) {

    let trimmed = dir.to_string() + "/" + path.trim_left_matches('/');
    println!("User requsted: {}", trimmed);

    // check if the file exists
    let mut file = match File::open(trimmed) {
        Ok(file) => { file },
        Err(e) => {
            println!("Error: {}\n Error: {}", path, e);
            let res = b"HTTP/1.0 404 Not Found\n \
                        Connection: close\n";
            let _ = stream.write(res);
            return;
        }
    };

    // read file into a new vector
    let mut contents = Vec::new();
    let _ = file.read_to_end(&mut contents);

    // construct response for the client
    let now = time::now();
    let res = format!("HTTP/1.0 200 OK\n\
                       Date: {}\n\
                       Content-Length: {}\n\
                       Connection: close\n\r\n", now.ctime(), contents.len());

    // send the header and the data
    let _ = stream.write(&*res.into_bytes());
    let _ = stream.write(&*contents);
    
    // ensure all data has been sent to the client
    stream.flush().unwrap();
}

/// usage
/// Simple usage printer that will be called in the case of invalid arguments
fn usage() {
    println!("Usage: ./erode [-dlh] -h HOST -p PORT");
    process::exit(0);
}

/// Main
///
fn main() {

    // grab and parse the arguments
    let args: Vec<_> = env::args().collect();
    let mut opts = Options::new();
    opts.optflag("d", "daemon", "Daemonize the server");
    opts.optflag("l", "log", "Turn on logging to the console");
    opts.reqopt("a", "address", "Local address to bind to", "ADDRESS");
    opts.reqopt("p", "port", "Local port to bind to", "PORT");
    opts.reqopt("r", "directory", "Directory to be served", "DIRECTORY");
    opts.optflag("h", "help", "Print help");

    //parse command line arguemnts
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m }
        Err(f) => { panic!(f.to_string()) }
    };

    if matches.opt_present("h") {
        usage();
        return;
    }

    // combine the strings into the addrress
    let address = matches.opt_str("a").unwrap();
    let port = matches.opt_str("p").unwrap();


    let r = match matches.opt_str("r") {
        Some(dir) => dir,
        None => ".".to_string()
    };

    let addr = address.to_string() + ":" + &port;

    // Bind a new TCP Socket to the given address
    let listener = match TcpListener::bind(&*addr) {
        Ok(listener) => {
            listener
        },
        Err(e) => {
            println!("Error: {}", e);
            return;
        }
    };

    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        let dir = r.clone();
        match stream {
            Ok(stream) => {
                thread::spawn(move|| {
                    handle_client(stream, &dir);
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

#[test]
#[should_panic]
fn it_works() {
    assert_eq!("hello", "world"); 
}
