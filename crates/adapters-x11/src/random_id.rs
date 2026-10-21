//! 衝突しないアカウント ID（FR-009）。`/dev/urandom` の 16 バイト。

use std::fs;

use nene_docket_core::account_id::AccountId;

use crate::adapter_failure::AdapterFailure;

/// 乱数の出どころ。
const SOURCE: &str = "/dev/urandom";

/// 新しい ID。
pub(crate) fn fresh() -> Result<AccountId, AdapterFailure> {
    let bytes = read_sixteen()?;
    Ok(AccountId::new(u128::from_be_bytes(bytes)))
}

fn read_sixteen() -> Result<[u8; 16], AdapterFailure> {
    use std::io::Read as _;
    let mut bytes = [0_u8; 16];
    fs::File::open(SOURCE)?.read_exact(&mut bytes)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_fresh_ids_differ() {
        let first = fresh().unwrap();
        let second = fresh().unwrap();
        assert_ne!(first, second);
        assert_eq!(first.hex().len(), 32);
    }
}
