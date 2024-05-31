/// Configuration module for the application.
/// The configuration can be parsed from the command-line arguments.
/// The configuration includes the port the server will listen on and the replica-of address.
/// If the replica-of address is set, the server will act as a replica of the given address.
///
/// To parse the configuration from the command-line arguments, use the `from_command_line` function.
/// The function returns a `Result` with the `Config` struct or an error message.
/// ```rs
/// use config::{Config, from_command_line};
///
/// let args: Vec<String> = std::env::args().collect(); // Get the command-line arguments
/// let config = from_command_line(args).expect("Failed to parse command-line arguments"); // Parse the configuration
///
/// // Print the configuration values
/// println!("Port: {}", config.port);
/// if let Some(replicaof) = &config.replicaof {
///    println!("Replica-of: {}", replicaof);
/// }
/// ```

// -------------
// CONFIGURATION
// -------------

/// The default port the server will listen on.
const DEFAULT_PORT: u16 = 6379;

/// Configuration for the application.
pub struct Config {
    /// The port the server will listen on. (Defaults to 6379)
    pub port: u16,

    /// The replica-of address.
    /// If set, the server will act as a replica of the given address.
    pub replicaof: Option<String>,

    /// The directory where the server will store the database files.
    pub dir: Option<String>,

    /// The filename of the database file.
    pub dbfilename: Option<String>,
}

/// Default implementation for the Config struct.
/// The default port is 6379 and there is no replica-of address.
impl Default for Config {
    fn default() -> Self {
        Config {
            port: DEFAULT_PORT, // Default port. Same as Redis.
            replicaof: None, // No replica-of address by default. The server will act as a master.
            dir: None,       // Default directory for the database files.
            dbfilename: None, // Default filename for the database file.
        }
    }
}

/// Parses the Configuration from the command-line arguments.
pub fn from_command_line(args: Vec<String>) -> Result<Config, Box<dyn std::error::Error>> {
    // Initialize the configuration with the default values
    let mut config = Config::default();

    // Iterate over the arguments...
    for i in 0..args.len() {
        match args[i].as_str() {
            // If the argument is a port flag, parse the port
            "-p" | "--port" => config.port = parse_port(&args, i)?,

            // If the argument is a replica-of flag, parse the replica-of address
            "--replicaof" => config.replicaof = parse_replicaof(&args, i)?,

            // If the argument is a directory flag, parse the directory
            "--dir" => config.dir = parse_dir(i, &args)?,

            // If the argument is a dbfilename flag, parse the dbfilename
            "--dbfilename" => config.dbfilename = parse_dbfilename(i, &args)?,

            _ => {} // Ignore any other arguments
        }
    }

    Ok(config)
}

// PORT
// ----

/// Parses the port from the command-line arguments.
/// The port must be specified in the format `--port 1234`.
fn parse_port(args: &[String], idx: usize) -> Result<u16, Box<dyn std::error::Error>> {
    // Check if there is a value after the flag...
    if idx + 1 < args.len() {
        // ...and if there is, parse it as a u16
        return Ok(args[idx + 1].parse::<u16>()?);
    } else {
        // ...otherwise, print an error message
        Err("No port provided after the flag")?;
    }
    Ok(DEFAULT_PORT)
}

// REPLICA-OF
// ----------

/// Parses the replica-of address from the command-line arguments.
/// The replica-of address must be specified in the format `--replicaof 'host port'`.
fn parse_replicaof(
    args: &[String],
    idx: usize,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    // Check if there is a value after the flag...
    if idx + 1 < args.len() {
        // ...and if there is, parse it as a string
        let str = args[idx + 1].clone();
        // Split the string into host and port parts (the --replicaof format is 'host port')
        let parts: Vec<&str> = str.split(' ').collect();
        let addr = format!("{}:{}", parts[0], parts[1]); // Combine the parts into an address
        return Ok(Some(addr));
    } else {
        // ...otherwise, print an error message
        Err("No replica-of address provided after the flag")?;
    }
    Ok(None)
}

// DIR
// ---

/// Parses the directory from the command-line arguments.
/// The directory must be specified in the format `--dir 'path'`.
/// The directory is where the server will store the rdb database files.
fn parse_dir(i: usize, args: &Vec<String>) -> Result<Option<String>, Box<dyn std::error::Error>> {
    // Check if there is a value after the flag...
    if i + 1 < args.len() {
        // ...and if there is, set it as the directory
        let dir = args[i + 1].clone();
        return Ok(Some(dir));
    } else {
        // ...otherwise, print an error message
        Err("No directory provided after the flag")?;
    }
    Ok(None)
}

// DBFILENAME
// ----------

pub fn parse_dbfilename(
    i: usize,
    args: &Vec<String>,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    // Check if there is a value after the flag...
    if i + 1 < args.len() {
        // ...and if there is, set it as the dbfilename
        let dbfilename = args[i + 1].clone();
        return Ok(Some(dbfilename));
    } else {
        // ...otherwise, print an error message
        Err("No dbfilename provided after the flag")?;
    }
    Ok(None)
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
    fn should_parse_dir() {
        let args: Vec<String> = vec!["--dir".into(), "/data/tmp".into()];
        let cli = from_command_line(args).unwrap();
        assert_eq!(cli.dir, Some("/data/tmp".into()));
    }

    #[test]
    fn should_parse_dbfilename() {
        let args: Vec<String> = vec!["--dbfilename".into(), "dump.rdb".into()];
        let cli = from_command_line(args).unwrap();
        assert_eq!(cli.dbfilename, Some("dump.rdb".into()));
    }

    #[test]
    fn should_ignore_any_other_arguments() {
        let args: Vec<String> = vec!["--port".into(), "2142".into(), "--foo".into(), "bar".into()];
        let cli = from_command_line(args).unwrap();
        assert_eq!(cli.port, 2142);
    }

    #[test]
    fn should_error_if_no_port_value() {
        let args: Vec<String> = vec!["--port".into()];
        match from_command_line(args) {
            Ok(_) => panic!("Should have errored"),
            Err(e) => assert_eq!(e.to_string(), "No port provided after the flag"),
        }
    }

    #[test]
    fn should_error_if_no_replicaof_value() {
        let args: Vec<String> = vec!["--replicaof".into()];
        match from_command_line(args) {
            Ok(_) => panic!("Should have errored"),
            Err(e) => assert_eq!(
                e.to_string(),
                "No replica-of address provided after the flag"
            ),
        }
    }
}
