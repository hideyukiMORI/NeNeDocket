//! 衝突しないアカウント ID と、署名する鍵（FR-009 / FR-010）。`BCryptGenRandom` の 16 バイトと 32 バイト。

use nene_docket_core::account_id::AccountId;
use nene_docket_core::signing_key::SigningKey;
use windows::Win32::Security::Cryptography::{BCRYPT_USE_SYSTEM_PREFERRED_RNG, BCryptGenRandom};

use crate::adapter_failure::AdapterFailure;

/// 新しい ID。
pub(crate) fn fresh() -> Result<AccountId, AdapterFailure> {
    Ok(AccountId::new(u128::from_be_bytes(read()?)))
}

/// 新しい鍵（ADR 0010）。
pub(crate) fn fresh_key() -> Result<SigningKey, AdapterFailure> {
    Ok(SigningKey::from_bytes(&read()?))
}

#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の BCryptGenRandom を呼ぶ（ADR 0002 の 4）"
)]
fn read<const N: usize>() -> Result<[u8; N], AdapterFailure> {
    let mut bytes = [0_u8; N];
    // SAFETY: bytes はこの関数の局所で、呼び出しの間だけ書かれる。系統の乱数源を使う。
    unsafe { BCryptGenRandom(None, &mut bytes, BCRYPT_USE_SYSTEM_PREFERRED_RNG) }
        .ok()
        .map_err(AdapterFailure::Random)?;
    Ok(bytes)
}
