// ------------
// COMMAND LINE
// ------------

/// Struct to hold the command-line arguments
pub struct CommandLineArguments {
    /// The port to listen on
    pub port: u16,
    /// The replica-of address. If set, the server will act as a replica of the given address
    pub replicaof: Option<String>,
}

/// Default implementation for the CommandLineArguments struct
impl Default for CommandLineArguments {
    fn default() -> Self {
        CommandLineArguments {
            port: 6379,
            replicaof: None,
        }
    }
}

impl CommandLineArguments {
    /// Parses the command-line arguments
    pub fn parse(&mut self, args: Vec<String>) -> &Self {
        for i in 0..args.len() {
            match args[i].as_str() {
                "-p" | "--port" => {
                    if i + 1 < args.len() {
                        match args[i + 1].parse::<u16>() {
                            Ok(port) => self.port = port,
                            Err(_) => {}
                        }
                    }
                }
                "--replicaof" => {
                    if i + 1 < args.len() {
                        self.replicaof = Some(args[i + 1].clone());
                    }
                }
                _ => {}
            }
        }
        self
    }
}
