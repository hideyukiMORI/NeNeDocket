//! Ed25519 の秘密鍵（32 バイト・FR-010・ADR 0010）。設定フォルダの `account.v2.txt` に 16 進 64 桁で置く。
//!
//! 鍵の乱数はアダプタが読む（ARC-007）。ここは与えられたバイト列から鍵を作り、署名するだけ。

use alloc::string::String;

use ed25519_dalek::Signer as _;

use crate::hex;
use crate::public_key::PublicKey;
use crate::signature::Signature;

/// 鍵の長さ（バイト）。
const LEN: usize = 32;

/// 署名する側の鍵。`Debug` は dalek の実装で、秘密のバイト列を出さない。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SigningKey {
    inner: ed25519_dalek::SigningKey,
}

impl SigningKey {
    /// 16 進の桁数。
    pub const HEX_LEN: usize = LEN * 2;

    /// 32 バイトの乱数から。どのバイト列も鍵になる。
    #[must_use]
    pub fn from_bytes(bytes: &[u8; LEN]) -> Self {
        Self {
            inner: ed25519_dalek::SigningKey::from_bytes(bytes),
        }
    }

    /// 16 進 64 桁から。桁数が違う・16 進でないなら `None`。
    #[must_use]
    pub fn from_hex(text: &str) -> Option<Self> {
        hex::decode::<LEN>(text).map(|bytes| Self::from_bytes(&bytes))
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

    /// 対になる公開鍵。
    #[must_use]
    pub fn public(&self) -> PublicKey {
        PublicKey::wrap(self.inner.verifying_key())
    }

    /// `message` に署名する。決定的（同じ鍵と同じ文には同じ署名）。
    #[must_use]
    pub fn sign(&self, message: &[u8]) -> Signature {
        Signature::wrap(self.inner.sign(message))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_signature_verifies_with_the_public_key_and_nothing_else() {
        let key = SigningKey::from_bytes(&[9_u8; LEN]);
        let other = SigningKey::from_bytes(&[10_u8; LEN]);
        let signature = key.sign(b"v2\t1");
        assert_eq!(signature, key.sign(b"v2\t1"));
        assert!(key.public().verify(b"v2\t1", &signature));
        assert!(!key.public().verify(b"v2\t2", &signature));
        assert!(!other.public().verify(b"v2\t1", &signature));
        let mut bytes = signature.to_bytes();
        bytes[0] ^= 1;
        assert!(
            !key.public()
                .verify(b"v2\t1", &Signature::from_bytes(&bytes))
        );
    }

    #[test]
    fn hex_round_trips_and_the_debug_form_hides_the_secret() {
        let key = SigningKey::from_bytes(&[9_u8; LEN]);
        assert_eq!(key.hex().len(), SigningKey::HEX_LEN);
        assert_eq!(SigningKey::from_hex(&key.hex()), Some(key.clone()));
        assert_eq!(SigningKey::from_hex("09"), None);
        assert!(!alloc::format!("{key:?}").contains(&key.hex()));
    }
}
