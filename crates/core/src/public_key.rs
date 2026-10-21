//! Ed25519 の公開鍵（32 バイト・FR-010・ADR 0010）。台帳の `keys/<account>.v1.pub` に 16 進 64 桁で置く。

use alloc::string::String;

use crate::hex;
use crate::signature::Signature;

/// 鍵の長さ（バイト）。
const LEN: usize = 32;

/// 検証する側の鍵。署名する側は [`crate::signing_key::SigningKey`]。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublicKey {
    inner: ed25519_dalek::VerifyingKey,
}

impl PublicKey {
    /// 16 進の桁数。
    pub const HEX_LEN: usize = LEN * 2;

    /// dalek の値を包む。
    pub(crate) const fn wrap(inner: ed25519_dalek::VerifyingKey) -> Self {
        Self { inner }
    }

    /// 32 バイトから。曲線の点にならないバイト列は `None`。
    #[must_use]
    pub fn from_bytes(bytes: &[u8; LEN]) -> Option<Self> {
        ed25519_dalek::VerifyingKey::from_bytes(bytes)
            .ok()
            .map(Self::wrap)
    }

    /// 16 進 64 桁から。桁数が違う・16 進でない・点にならないなら `None`。
    #[must_use]
    pub fn from_hex(text: &str) -> Option<Self> {
        hex::decode::<LEN>(text).and_then(|bytes| Self::from_bytes(&bytes))
    }

    /// 32 バイト。
    #[must_use]
    pub fn to_bytes(&self) -> [u8; LEN] {
        self.inner.to_bytes()
    }

    /// 16 進 64 桁（小文字）。
    #[must_use]
    pub fn hex(&self) -> String {
        hex::encode(&self.to_bytes())
    }

    /// `message` への `signature` がこの鍵の持ち主のものか。厳密な検証（可鍛性のある署名も拒む）。
    #[must_use]
    pub fn verify(&self, message: &[u8], signature: &Signature) -> bool {
        self.inner.verify_strict(message, signature.inner()).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signing_key::SigningKey;

    #[test]
    fn hex_round_trips_and_bad_text_is_refused() {
        let key = SigningKey::from_bytes(&[3_u8; LEN]).public();
        assert_eq!(key.hex().len(), PublicKey::HEX_LEN);
        assert_eq!(PublicKey::from_hex(&key.hex()), Some(key));
        assert_eq!(PublicKey::from_bytes(&key.to_bytes()), Some(key));
        assert_eq!(PublicKey::from_hex("abc"), None);
        assert_eq!(PublicKey::from_hex(&"g".repeat(PublicKey::HEX_LEN)), None);
    }
}
