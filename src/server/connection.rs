// Library
use crate::{
    commands,
    parser::{self, resp},
    server::Server,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{mpsc, Mutex},
};

// ----------
// CONNECTION
// ----------

/// The size of the buffer to read incoming data
const BUFFER_SIZE: usize = 1024;

/// Represents a connection to a client.
/// Contains the stream, the address of the client, and a buffer to store incoming data.
/// This struct is used to store and handle the connection information for each client.
/// The server will create a new Connection instance for each incoming connection.
pub struct Connection {
    /// The TcpStream used to communicate with the client.
    /// The stream is used to read and write data to the client.
    /// The stream is created when a new connection is accepted by the server.
    /// The stream is closed when the connection is closed.
    pub stream: TcpStream,

    /// The address of the client.
    /// Contains the IP address and port number of the client.
    /// The address is used to identify the client and send responses back to the client.
    /// The address is set when a new connection is accepted by the server.
    pub addr: SocketAddr,

    /// The buffer used to store incoming data from the client.
    /// The buffer is used to read data from the stream and process it.
    /// The buffer is cleared after each request is processed.
    buffer: [u8; BUFFER_SIZE],

    /// The kind of connection (Main or Replication)
    /// The role is used to determine the type of connection (master or replica).
    /// The role is set when the connection is created.
    pub kind: Kind,
}

/// The kind of connection (Main or Replication)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Kind {
    Main,
    Replication,
}

/// Instantiate a new Connection with the provided TcpStream and SocketAddr.
pub fn new(stream: TcpStream, addr: SocketAddr, kind: Kind) -> Connection {
    Connection {
        stream,
        addr,
        buffer: [0; BUFFER_SIZE],
        kind,
    }
}

// Implementation of the Connection struct
impl Connection {
    /// Reads data from the stream and stores it in the buffer.
    /// The read_data method is called when the server receives data from the client.
    /// The server will read the data from the stream and store it in the buffer.
    /// The buffer is used to process the incoming data and generate a response.
    pub async fn read(&mut self) -> Result<usize, Box<dyn std::error::Error>> {
        let bytes_read = self.stream.read(&mut self.buffer).await?;
        Ok(bytes_read)
    }

    /// Writes data to the stream.
    /// The write_data method is called when the server needs to send a response to the client.
    /// The server will write the response to the stream, which will be sent to the client.
    pub async fn write_all(&mut self, data: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        self.stream.write_all(data).await?;
        self.stream.flush().await?;
        Ok(())
    }

    /// Returns a slice of the buffer containing the read data.
    /// The read_buffer method should be called after reading data from the stream.
    ///
    /// ```rs
    /// let bytes_read = connection.read().await?;
    /// let buffer = connection.read_buffer(bytes_read);
    /// ```
    pub fn read_buffer(&self, len: usize) -> &[u8] {
        &self.buffer[..len]
    }

    // /// Parses the buffer and returns the data as a string.
    // pub fn parse_from_buffer(&mut self) -> String {
    //     String::from_utf8_lossy(&self.buffer).to_string()
    //     // self.buffer = [0; BUFFER_SIZE]; // Clear the buffer
    // }

    // /// Clears the buffer by setting all elements to 0.
    // pub fn clear_buffer(&mut self) {
    //     self.buffer = [0; BUFFER_SIZE];
    // }

    /// Handles the incoming connection stream by reading the incoming data,
    /// parsing it, and writing a response back to the stream.
    pub async fn handle(
        &mut self,
        server: &Arc<Mutex<Server>>,
        wait_channel: &Arc<Mutex<(mpsc::Sender<u64>, mpsc::Receiver<u64>)>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("New connection from {}", self.addr);
        loop {
            // Read the incoming data from the stream
            let bytes_read = self.read().await?;
            println!("Bytes read: {}", bytes_read);
            if bytes_read == 0 {
                // If no data was read, this typically indicates that the end of the
                // stream has been reached and the connection should be closed.
                break;
            }

            // Parse the incoming data
            let request = self.read_buffer(bytes_read);
            let len = request.len();
            // println!("Received: {:?}", String::from_utf8_lossy(request));

            let mut err_response: Option<String> = None;
            let mut cmds: Vec<parser::resp::Type> = Vec::new();
            match parser::parse(request) {
                Ok(c) => cmds = c,
                Err(e) => {
                    err_response = Some(format!("-ERR {}\r\n", e));
                }
            }
            // println!("Parsed: {:?} of len", cmds);

            if let Some(r) = err_response {
                self.write_all(r.as_bytes()).await?;
                continue;
            }

            // Iterate over the parsed commands
            // There can be multiple commands in a single request
            for cmd in cmds {
                match cmd {
                    resp::Type::Array(command) => {
                        println!("Array: {:?}", command);
                        commands::handle(&command, self, server, wait_channel).await?;
                        let mut server = server.lock().await;
                        println!(
                            "repl_offset: {}, mater_repl_offset: {}",
                            server.repl_offset, server.master_repl_offset
                        );
                        match &command[0] {
                            resp::Type::BulkString(ref cmd) => {
                                if cmd.to_uppercase() == "SET" {
                                    if !server.role.is_master() {
                                        println!("{} {} {}", cmd, server.repl_offset, len as u64);
                                        server.repl_offset += len as u64;
                                    } else {
                                        println!("{} {} {}", cmd, server.repl_offset, len as u64);
                                        server.master_repl_offset += len as u64;
                                    }
                                } else if cmd.to_uppercase() == "PING" {
                                    if !server.role.is_master() {
                                        println!("{} {} {}", cmd, server.repl_offset, len as u64);
                                        server.repl_offset += len as u64;
                                    }
                                } else if cmd.to_uppercase() == "REPLCONF" {
                                    match &command[1] {
                                        resp::Type::BulkString(subcommand) => {
                                            if subcommand.to_uppercase() == "GETACK" {
                                                if !server.role.is_master() {
                                                    println!(
                                                        "{} {} {}",
                                                        cmd, server.repl_offset, len as u64
                                                    );
                                                    server.repl_offset += len as u64;
                                                }
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            _ => {}
                        }
                        println!(
                            "repl_offset: {}, mater_repl_offset: {}",
                            server.repl_offset, server.master_repl_offset
                        );
                    }
                    resp::Type::RDBFile(_data) => {
                        // let response =
                        //     resp::Type::Array(vec![resp::Type::SimpleString("OK".into())]);
                        // self.write_all(&response.as_bytes()).await?;
                        continue;
                    }
                    _ => {
                        let response = resp::Type::SimpleError("ERR unknown command\r\n".into());
                        self.write_all(&response.as_bytes()).await?;
                    }
                }
            }
        }
        println!("Connection closed for {}", self.addr);

        // Once we are out of the loop, the connection will be closed.
        Ok(())
    }
}
