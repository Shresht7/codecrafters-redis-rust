// -------------
// CONFIGURATION
// -------------

/// Configuration for the application.
pub struct Config {
    /// The port the server will listen on. (Defaults to 6379)
    pub port: u16,

    /// The replica-of address.
    /// If set, the server will act as a replica of the given address.
    pub replicaof: Option<String>,
}

/// Default implementation for the Config struct.
/// The default port is 6379 and there is no replica-of address.
impl Default for Config {
    fn default() -> Self {
        Config {
            port: 6379,      // Default port. Same as Redis.
            replicaof: None, // No replica-of address by default. The server will act as a master.
        }
    }
}

/// Parses the Configuration from the command-line arguments.
pub fn from_command_line(args: Vec<String>) -> Result<Config, Box<dyn std::error::Error>> {
    let mut config = Config::default();

    for i in 0..args.len() {
        match args[i].as_str() {
            "-p" | "--port" => {
                if i + 1 < args.len() {
                    config.port = args[i + 1].parse::<u16>()?;
                }
            }
            "--replicaof" => {
                if i + 1 < args.len() {
                    let str = args[i + 1].clone();
                    // Split the string into host and port
                    let parts: Vec<&str> = str.split(':').collect();
                    let addr = format!("{}:{}", parts[0], parts[1]);
                    config.replicaof = Some(addr);
                }
            }
            _ => {}
        }
    }

    Ok(config)
}

// -----
// TESTS
// -----

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_defaults_if_there_is_nothing_to_parse() {
        let args: Vec<String> = vec![];
        let cli = from_command_line(args).unwrap();
        assert_eq!(cli.port, 6379);
        assert_eq!(cli.replicaof, None);
    }

    #[test]
    fn should_parse_port() {
        let args: Vec<String> = vec!["--port".into(), "4321".into()];
        let cli = from_command_line(args).unwrap();
        assert_eq!(cli.port, 4321);
    }

    #[test]
    fn should_parse_shorthand_port() {
        let args: Vec<String> = vec!["-p".into(), "5678".into()];
        let cli = from_command_line(args).unwrap();
        assert_eq!(cli.port, 5678);
    }

    #[test]
    fn should_parse_replicaof() {
        let args: Vec<String> = vec!["--replicaof".into(), "111.222.333.444 9876".into()];
        let cli = from_command_line(args).unwrap();
        assert_eq!(cli.replicaof, Some("111.222.333.444:9876".into()));
    }

    #[test]
    fn should_parse_both_port_and_replicaof() {
        let args: Vec<String> = vec![
            "--port".into(),
            "5000".into(),
            "--replicaof".into(),
            "111.222.333.444 3000".into(),
        ];
        let cli = from_command_line(args).unwrap();
        assert_eq!(cli.port, 5000);
        assert_eq!(cli.replicaof, Some("111.222.333.444:3000".into()));
    }

    #[test]
    fn should_ignore_any_other_arguments() {
        let args: Vec<String> = vec!["--port".into(), "2142".into(), "--foo".into(), "bar".into()];
        let cli = from_command_line(args).unwrap();
        assert_eq!(cli.port, 2142);
    }
}
