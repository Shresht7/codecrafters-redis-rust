// Library
use std::io::{Read, Write};
use std::net::TcpListener;

fn main() {
    // Create a TCPListener and bind it to the given address and port
    // Note: 6379 is the default port that Redis uses (You may have to stop any running Redis instances)
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    // Listen for incoming connections and handle them
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                handle_connection(&mut stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

/// Handles the incoming connection stream by reading the incoming data,
/// parsing it, and writing a response back to the stream.
fn handle_connection(stream: &mut std::net::TcpStream) {
    /// The size of the buffer to read incoming data
    const BUFFER_SIZE: usize = 1024;

    // Read the incoming data from the stream
    let mut buffer = [0; BUFFER_SIZE];
    stream.read(&mut buffer).unwrap();

    // Print the incoming data
    println!("Request: {}", String::from_utf8_lossy(&buffer));

    // Write a response back to the stream
    let response = "+PONG\r\n";
    stream.write_all(response.as_bytes()).unwrap();

    // Flush the stream to ensure the response is sent
    stream.flush().unwrap();
}
