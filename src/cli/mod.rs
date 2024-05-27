// ------------
// COMMAND LINE
// ------------

/// Struct to hold the command-line arguments
pub struct CommandLineArguments {
    pub port: u16,
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
                _ => {}
            }
        }
        self
    }
}
