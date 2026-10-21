//! 衝突しないアカウント ID と、署名する鍵（FR-009 / FR-010）。`/dev/urandom` の 16 バイトと 32 バイト。

use std::fs;

use nene_docket_core::account_id::AccountId;
use nene_docket_core::signing_key::SigningKey;

use crate::adapter_failure::AdapterFailure;

/// 乱数の出どころ。
const SOURCE: &str = "/dev/urandom";

/// 新しい ID。
pub(crate) fn fresh() -> Result<AccountId, AdapterFailure> {
    Ok(AccountId::new(u128::from_be_bytes(read()?)))
}

/// 新しい鍵（ADR 0010）。
pub(crate) fn fresh_key() -> Result<SigningKey, AdapterFailure> {
    Ok(SigningKey::from_bytes(&read()?))
}

fn read<const N: usize>() -> Result<[u8; N], AdapterFailure> {
    use std::io::Read as _;
    let mut bytes = [0_u8; N];
    fs::File::open(SOURCE)?.read_exact(&mut bytes)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_fresh_ids_and_keys_differ() {
        let first = fresh().unwrap();
        let second = fresh().unwrap();
        assert_ne!(first, second);
        assert_eq!(first.hex().len(), 32);
        assert_ne!(fresh_key().unwrap(), fresh_key().unwrap());
    }
}
