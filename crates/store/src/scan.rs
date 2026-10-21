//! 他人のジャーナルの 1 回の読みの結果（Issue #75）。`scanner` のスレッドが作り、`file_journal` が取り込む。

use nene_docket_application::key_record::KeyRecord;

use crate::other_journals::{LocatedOp, OtherJournals};
use crate::store_failure::StoreFailure;
use crate::verifier::Verifier;

/// 1 回の読みの結果: 届いた操作・読み位置と壊れた行の数を更新した `OtherJournals`・読み直した鍵と検証器・失敗。
#[derive(Debug)]
pub(crate) struct Scan {
    fresh: Vec<LocatedOp>,
    others: OtherJournals,
    keys: Vec<KeyRecord>,
    verifier: Option<Verifier>,
    failure: Option<StoreFailure>,
}

impl Scan {
    /// 値から。
    pub(crate) fn new(
        (fresh, failure): (Vec<LocatedOp>, Option<StoreFailure>),
        (others, keys, verifier): (OtherJournals, Vec<KeyRecord>, Option<Verifier>),
    ) -> Self {
        Self {
            fresh,
            others,
            keys,
            verifier,
            failure,
        }
    }

    /// 届いた操作と失敗、読み位置・鍵・検証器に分ける。
    pub(crate) fn into_parts(self) -> ScanParts {
        (
            (self.fresh, self.failure),
            (self.others, self.keys, self.verifier),
        )
    }
}

/// `Scan` の中身: （届いた操作, 失敗）と（読み位置, 鍵, 検証器）。
pub(crate) type ScanParts = (
    (Vec<LocatedOp>, Option<StoreFailure>),
    (OtherJournals, Vec<KeyRecord>, Option<Verifier>),
);
