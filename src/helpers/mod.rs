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
}
