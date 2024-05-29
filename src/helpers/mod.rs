// Library
use rand::Rng;

// ----------------
// HELPER FUNCTIONS
// ----------------

/// Generate a pseudo-random string of given length using alphanumeric characters.
pub fn generate_id(len: u16) -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    let id: String = (0..len)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();
    id
}

/// Split the host and port from the given address.
/// The address should be in the format "host:port".
///
/// ```rs
/// let addr = "127.0.0.1:6379";
/// let (host, port) = split_host_and_port(addr).unwrap();
/// assert_eq!(host, "127.0.0.1"); // true
/// assert_eq!(port, 6379); // true
/// ```
pub fn split_host_and_port(
    addr: String,
    sep: &str,
) -> Result<(String, u16), Box<dyn std::error::Error>> {
    // Check if the address contains the separator
    if !addr.contains(sep) {
        return Err("Invalid address".into());
    }
    // Split the address into host and port
    let parts: Vec<&str> = addr.split(sep).collect();

    // Get the host and port parts
    let host = parts.get(0).ok_or("Invalid address")?;
    let port = parts.get(1).ok_or("Invalid address")?;
    let port = port.parse::<u16>()?; // Convert port to u16

    // Return the host and port
    Ok((host.to_string(), port))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_generate_random_id() {
        let id1 = generate_id(40);
        let id2 = generate_id(40);
        assert_ne!(id1, id2);
    }

    #[test]
    fn should_generate_id_of_given_length() {
        let id1 = generate_id(40);
        assert_eq!(id1.len(), 40);
        let id2 = generate_id(24);
        assert_eq!(id2.len(), 24);
    }

    #[test]
    fn should_split_host_and_port() {
        let addr = "127.0.0.1:6379".into();
        let (host, port) = split_host_and_port(addr, ":").unwrap();
        assert_eq!(host, "127.0.0.1");
        assert_eq!(port, 6379);
    }

    #[test]
    fn should_support_different_separators() {
        let addr = "127.0.0.1 6379".into();
        let (host, port) = split_host_and_port(addr, " ").unwrap();
        assert_eq!(host, "127.0.0.1");
        assert_eq!(port, 6379);
    }

    #[test]
    fn should_fail_to_split_host_and_port() {
        let addr = "127.0.0.1".into();
        let result = split_host_and_port(addr, ":");
        assert!(result.is_err());
    }

    #[test]
    fn should_fail_to_split_host_and_port_with_invalid_port() {
        let addr = "127.0.0.1:invalid".into();
        let result = split_host_and_port(addr, ":");
        assert!(result.is_err());
    }
}
