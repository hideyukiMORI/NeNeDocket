//! 衝突しないアカウント ID（FR-009）。`BCryptGenRandom` の 16 バイト。

use nene_docket_core::account_id::AccountId;
use windows::Win32::Security::Cryptography::{BCRYPT_USE_SYSTEM_PREFERRED_RNG, BCryptGenRandom};

use crate::adapter_failure::AdapterFailure;

/// 新しい ID。
#[expect(
    unsafe_code,
    reason = "RS-010: Win32 の BCryptGenRandom を呼ぶ（ADR 0002 の 4）"
)]
pub(crate) fn fresh() -> Result<AccountId, AdapterFailure> {
    let mut bytes = [0_u8; 16];
    // SAFETY: bytes はこの関数の局所で、呼び出しの間だけ書かれる。系統の乱数源を使う。
    unsafe { BCryptGenRandom(None, &mut bytes, BCRYPT_USE_SYSTEM_PREFERRED_RNG) }
        .ok()
        .map_err(AdapterFailure::Random)?;
    Ok(AccountId::new(u128::from_be_bytes(bytes)))
}
