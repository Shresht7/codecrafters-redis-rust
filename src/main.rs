// Library
use std::net::TcpListener;

fn main() {
    // Create a TCPListener and bind it to the given address and port
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    // Listen for incoming connections and handle them
    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
