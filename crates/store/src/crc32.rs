//! CRC-32（IEEE 802.3）。レコードの末尾の検査和。表を持たず 1 ビットずつ回す（依存 0・速さは要らない）。

/// 多項式（反転形）。
const POLYNOMIAL: u32 = 0xEDB8_8320;

/// バイト列の CRC-32。
pub(crate) fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0_u8..8 {
            let low = crc & 1;
            crc >>= 1_u32;
            if low == 1 {
                crc ^= POLYNOMIAL;
            }
        }
    }
    !crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_the_well_known_check_value() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(crc32(b""), 0);
        assert_ne!(crc32(b"a"), crc32(b"b"));
    }
}
