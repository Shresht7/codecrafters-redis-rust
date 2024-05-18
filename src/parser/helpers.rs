// Library
use super::CRLF;

// ----------------
// HELPER FUNCTIONS
// ----------------

/// Finds the position of the next CRLF sequence in the input
pub fn find_crlf(input: &[u8]) -> Result<usize, Box<dyn std::error::Error>> {
    input
        .windows(CRLF.len())
        .position(|window| window == CRLF)
        .ok_or("Invalid input. Expecting a CRLF sequence".into())
}
