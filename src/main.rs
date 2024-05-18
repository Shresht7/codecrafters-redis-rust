// Library
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

fn main() {
    // Run the server on the given address and port
    run_server("127.0.0.1:6379").unwrap();
}

/// Runs the TCP server on the given address and port
fn run_server(addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create a TCPListener and bind it to the given address and port
    // Note: 6379 is the default port that Redis uses (You may have to stop any running Redis instances)
    let listener = TcpListener::bind(addr)?;

    // Listen for incoming connections and handle them
    for stream in listener.incoming() {
        // Unwrap the stream to get the TcpStream object and clone it to allow multiple connections
        let mut s = stream?.try_clone()?;

        // Set the stream to non-blocking mode to allow multiple connections to be handled concurrently
        // s.set_nonblocking(true)?;

        // Spawn a new thread for each incoming connection
        thread::spawn(move || {
            handle_connection(&mut s).expect("Failed to handle connection");
        });
    }

    Ok(())
}

/// Handles the incoming connection stream by reading the incoming data,
/// parsing it, and writing a response back to the stream.
fn handle_connection(stream: &mut std::net::TcpStream) -> Result<(), Box<dyn std::error::Error>> {
    /// The size of the buffer to read incoming data
    const BUFFER_SIZE: usize = 1024;

    // Loop as long as requests are being made
    loop {
        // Read the incoming data from the stream
        let mut buffer = [0; BUFFER_SIZE];
        let bytes_read = stream.read(&mut buffer)?;

        // If no bytes were read, the client has closed the connection
        if bytes_read == 0 {
            break;
        }

        // Print the incoming data
        println!("Request: {}", String::from_utf8_lossy(&buffer));

        // Write a response back to the stream
        let response = b"+PONG\r\n";
        stream.write_all(response)?;

        // Flush the stream to ensure the response is sent
        stream.flush()?;
    }

    Ok(())
}
