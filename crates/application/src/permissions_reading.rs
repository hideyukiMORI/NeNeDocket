//! `permissions.v1.txt` を読んだ結果。権限と署名と、持ち主の交代の直後なら署名した前の持ち主（ADR 0013）。

use nene_docket_core::account_id::AccountId;
use nene_docket_core::permissions::Permissions;
use nene_docket_core::signature::Signature;

/// 権限・署名・`handover`（前の持ち主の鍵で署名してあれば、その ID）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PermissionsReading {
    permissions: Permissions,
    signature: Signature,
    handover: Option<AccountId>,
}

impl PermissionsReading {
    /// 値から。
    #[must_use]
    pub const fn new(
        permissions: Permissions,
        signature: Signature,
        handover: Option<AccountId>,
    ) -> Self {
        Self {
            permissions,
            signature,
            handover,
        }
    }

    /// 権限。
    #[must_use]
    pub const fn permissions(&self) -> &Permissions {
        &self.permissions
    }

    /// 権限を取り出す。
    #[must_use]
    pub fn into_permissions(self) -> Permissions {
        self.permissions
    }

    /// 署名。
    #[must_use]
    pub const fn signature(&self) -> &Signature {
        &self.signature
    }

    /// 署名した前の持ち主（交代の直後だけ）。
    #[must_use]
    pub const fn handover(&self) -> Option<AccountId> {
        self.handover
    }
}
