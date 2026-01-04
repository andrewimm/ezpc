//! GDB Remote Serial Protocol packet handling
//!
//! Implements packet framing, checksums, and ACK/NAK protocol.
//! Format: $<data>#<checksum>
//! Checksum is 2-digit hex of sum(data) mod 256

/// Calculate GDB checksum (sum of bytes mod 256)
fn calculate_checksum(data: &str) -> u8 {
    data.bytes().fold(0u8, |acc, b| acc.wrapping_add(b))
}

/// Format data as GDB packet: $<data>#<checksum>
pub fn format_packet(data: &str) -> String {
    let checksum = calculate_checksum(data);
    format!("${}#{:02x}", data, checksum)
}

/// Parse raw bytes into GDB packet
/// Returns Some(data) if valid packet found, None otherwise
pub fn parse_packet(raw: &[u8]) -> Option<String> {
    // Find packet start '$'
    let start = raw.iter().position(|&b| b == b'$')?;

    // Find packet end '#'
    let end = raw[start..].iter().position(|&b| b == b'#')?;
    let hash_pos = start + end;

    // Need at least 2 more bytes for checksum
    if raw.len() < hash_pos + 3 {
        return None;
    }

    // Extract data portion (between $ and #)
    let data = std::str::from_utf8(&raw[start + 1..hash_pos]).ok()?;

    // Extract checksum (2 hex digits after #)
    let checksum_str = std::str::from_utf8(&raw[hash_pos + 1..hash_pos + 3]).ok()?;
    let expected_checksum = u8::from_str_radix(checksum_str, 16).ok()?;

    // Validate checksum
    let actual_checksum = calculate_checksum(data);
    if actual_checksum != expected_checksum {
        return None;
    }

    Some(data.to_string())
}

/// ACK byte (acknowledge valid packet)
pub const ACK: &[u8] = b"+";

/// NAK byte (request retransmission)
pub const NAK: &[u8] = b"-";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_checksum() {
        // q=0x71, S=0x53, u=0x75, p=0x70, p=0x70, o=0x6f, r=0x72, t=0x74, e=0x65, d=0x64
        // Sum = 0x437 = 1079, mod 256 = 55 = 0x37
        assert_eq!(calculate_checksum("qSupported"), 0x37);
        assert_eq!(calculate_checksum("g"), 0x67);
        assert_eq!(calculate_checksum(""), 0x00);
    }

    #[test]
    fn test_format_packet() {
        assert_eq!(format_packet("qSupported"), "$qSupported#37");
        assert_eq!(format_packet("g"), "$g#67");
        assert_eq!(format_packet(""), "$#00");
    }

    #[test]
    fn test_parse_packet_valid() {
        let raw = b"$qSupported#37";
        assert_eq!(parse_packet(raw), Some("qSupported".to_string()));

        let raw = b"$g#67";
        assert_eq!(parse_packet(raw), Some("g".to_string()));
    }

    #[test]
    fn test_parse_packet_invalid_checksum() {
        let raw = b"$qSupported#ff"; // Wrong checksum
        assert_eq!(parse_packet(raw), None);
    }

    #[test]
    fn test_parse_packet_incomplete() {
        let raw = b"$qSupported"; // Missing # and checksum
        assert_eq!(parse_packet(raw), None);

        let raw = b"$qSupported#"; // Missing checksum digits
        assert_eq!(parse_packet(raw), None);
    }

    #[test]
    fn test_parse_packet_with_prefix() {
        let raw = b"+$g#67"; // ACK before packet
        assert_eq!(parse_packet(raw), Some("g".to_string()));
    }
}
