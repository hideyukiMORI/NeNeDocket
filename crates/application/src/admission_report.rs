//! 署名と権限の検証の報告（FR-010・ADR 0010）。ジャーナルのポートが数え、画面が示す（FR-014）。

/// 捨てた行の数・自分の鍵の食い違い・自分が書けるか。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdmissionReport {
    unverified_lines: usize,
    key_mismatch: bool,
    may_edit: bool,
}

impl AdmissionReport {
    /// 権限を使わない台帳。何も捨てず、誰でも書ける。
    #[must_use]
    pub const fn open() -> Self {
        Self {
            unverified_lines: 0,
            key_mismatch: false,
            may_edit: true,
        }
    }

    /// 値から。
    #[must_use]
    pub const fn new(unverified_lines: usize, key_mismatch: bool, may_edit: bool) -> Self {
        Self {
            unverified_lines,
            key_mismatch,
            may_edit,
        }
    }

    /// 署名か権限が通らず捨てた行の数。
    #[must_use]
    pub const fn unverified_lines(&self) -> usize {
        self.unverified_lines
    }

    /// 台帳の `keys/` にある自分の公開鍵が手元の鍵と違う（同じアカウント ID の 2 台）。
    #[must_use]
    pub const fn key_mismatch(&self) -> bool {
        self.key_mismatch
    }

    /// 自分がこの台帳に書けるか。書けなくても追記はするが、読み手（自分を含む）が捨てる。
    #[must_use]
    pub const fn may_edit(&self) -> bool {
        self.may_edit
    }
}
