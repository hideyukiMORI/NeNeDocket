//! バイト列と 16 進（小文字）の相互変換。鍵と署名をファイルに書く形（ADR 0010）。

use alloc::string::String;

/// バイト列を 16 進（小文字）にする。
#[must_use]
pub fn encode(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        text.push(digit_char(byte >> 4_u8));
        text.push(digit_char(byte & 0x0F_u8));
    }
    text
}

/// 16 進を `N` バイトに戻す。桁数が `2N` でない・16 進でない（大文字も）なら `None`。
#[must_use]
pub fn decode<const N: usize>(text: &str) -> Option<[u8; N]> {
    let raw = text.as_bytes();
    if raw.len() != N.saturating_mul(2) {
        return None;
    }
    let mut bytes = [0_u8; N];
    let (pairs, _rest) = raw.as_chunks::<2>();
    for (slot, [high, low]) in bytes.iter_mut().zip(pairs) {
        *slot = (digit(*high)? << 4_u8) | digit(*low)?;
    }
    Some(bytes)
}

/// 0〜15 を小文字の 16 進 1 桁に。
fn digit_char(value: u8) -> char {
    char::from(match value {
        0..=9 => b'0' + value,
        _ => b'a' + (value - 10),
    })
}

/// 小文字の 16 進 1 桁の値。
fn digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_then_decode_round_trips_and_refuses_other_shapes() {
        let bytes = [0x00_u8, 0x0F, 0xA5, 0xFF];
        assert_eq!(encode(&bytes), "000fa5ff");
        assert_eq!(decode::<4>("000fa5ff"), Some(bytes));
        assert_eq!(decode::<4>("000FA5FF"), None);
        assert_eq!(decode::<4>("000fa5f"), None);
        assert_eq!(decode::<3>("000fa5ff"), None);
        assert_eq!(decode::<4>("000fa5fg"), None);
        assert_eq!(decode::<0>(""), Some([]));
        assert_eq!(encode(&[]), "");
    }
}
