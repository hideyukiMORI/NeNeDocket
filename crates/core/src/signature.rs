//! Ed25519 の署名（64 バイト・FR-010・ADR 0010）。ジャーナルの版 2 の行と `permissions.v1.txt` に 16 進 128 桁で書く。

use alloc::string::String;

use crate::hex;

/// 署名の長さ（バイト）。
const LEN: usize = 64;

/// 署名の値。作るのは [`crate::signing_key::SigningKey::sign`]、確かめるのは [`crate::public_key::PublicKey::verify`]。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signature {
    inner: ed25519_dalek::Signature,
}

impl Signature {
    /// 16 進の桁数。
    pub const HEX_LEN: usize = LEN * 2;

    /// dalek の値を包む。
    pub(crate) const fn wrap(inner: ed25519_dalek::Signature) -> Self {
        Self { inner }
    }

    /// 64 バイトから。
    #[must_use]
    pub fn from_bytes(bytes: &[u8; LEN]) -> Self {
        Self::wrap(ed25519_dalek::Signature::from_bytes(bytes))
    }

    /// 16 進 128 桁から。桁数が違う・16 進でないなら `None`。
    #[must_use]
    pub fn from_hex(text: &str) -> Option<Self> {
        hex::decode::<LEN>(text).map(|bytes| Self::from_bytes(&bytes))
    }

    /// 64 バイト。
    #[must_use]
    pub fn to_bytes(&self) -> [u8; LEN] {
        self.inner.to_bytes()
    }

    /// 16 進 128 桁（小文字）。
    #[must_use]
    pub fn hex(&self) -> String {
        hex::encode(&self.to_bytes())
    }

    /// dalek の値。
    pub(crate) const fn inner(&self) -> &ed25519_dalek::Signature {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_round_trips_and_wrong_lengths_are_refused() {
        let bytes = [7_u8; LEN];
        let signature = Signature::from_bytes(&bytes);
        assert_eq!(signature.hex().len(), Signature::HEX_LEN);
        assert_eq!(Signature::from_hex(&signature.hex()), Some(signature));
        assert_eq!(signature.to_bytes(), bytes);
        assert_eq!(Signature::from_hex("07"), None);
    }
}
