// Library
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
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
    stream: TcpStream,

    // TODO: Reimplement this
    // /// The address of the client.
    // /// Contains the IP address and port number of the client.
    // /// The address is used to identify the client and send responses back to the client.
    // /// The address is set when a new connection is accepted by the server.
    // addr: SocketAddr,
    /// The buffer used to store incoming data from the client.
    /// The buffer is used to read data from the stream and process it.
    /// The buffer is cleared after each request is processed.
    buffer: [u8; BUFFER_SIZE],
}

/// Instantiate a new Connection with the provided TcpStream and SocketAddr.
pub fn new(stream: TcpStream) -> Connection {
    Connection {
        stream,
        // addr, // TODO: Fix this
        buffer: [0; BUFFER_SIZE],
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
    // fn parse_from_buffer(&mut self) -> String {
    //     // Convert the buffer to a string
    //     let str = String::from_utf8_lossy(&self.buffer).to_string();
    //     self.buffer = [0; BUFFER_SIZE]; // Clear the buffer
    //     str
    // }
}