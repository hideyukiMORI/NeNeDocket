//! 台帳に参加したアカウント 1 人（FR-009）。`keys/` の名前と、権限から決まる役。

use alloc::string::String;
use alloc::vec::Vec;

use nene_docket_core::account_id::AccountId;
use nene_docket_core::permissions::Permissions;

use crate::member_role::MemberRole;

/// ID の先頭を出す桁数。
const ID_DIGITS: usize = 4;

/// ID・名前・役。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    id: AccountId,
    name: String,
    role: MemberRole,
}

impl Member {
    /// `keys/` の名前と権限（無ければ誰でも編集）から、ID の順に並べる。
    #[must_use]
    pub fn list(names: &[(AccountId, String)], permissions: Option<&Permissions>) -> Vec<Self> {
        let mut members: Vec<Self> = names
            .iter()
            .map(|(id, name)| Self {
                id: *id,
                name: name.clone(),
                role: role_of(*id, permissions),
            })
            .collect();
        members.sort_by_key(|member| member.id);
        members
    }

    /// ID。
    #[must_use]
    pub const fn id(&self) -> AccountId {
        self.id
    }

    /// 名前。
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// ID の先頭 4 桁。
    #[must_use]
    pub fn short_id(&self) -> String {
        self.id.hex().chars().take(ID_DIGITS).collect()
    }

    /// 役。
    #[must_use]
    pub const fn role(&self) -> MemberRole {
        self.role
    }
}

fn role_of(id: AccountId, permissions: Option<&Permissions>) -> MemberRole {
    match permissions {
        None => MemberRole::Anyone,
        Some(permissions) if permissions.owner() == id => MemberRole::Owner,
        Some(permissions) if permissions.may_edit(id) => MemberRole::Editor,
        Some(_) => MemberRole::Viewer,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OWNER: AccountId = AccountId::new(2);
    const EDITOR: AccountId = AccountId::new(1);
    const VIEWER: AccountId = AccountId::new(3);

    fn names() -> Vec<(AccountId, String)> {
        alloc::vec![
            (OWNER, String::from("o")),
            (VIEWER, String::from("v")),
            (EDITOR, String::from("e")),
        ]
    }

    #[test]
    fn roles_follow_the_permissions_and_the_list_is_by_id() {
        let permissions = Permissions::owner_only(OWNER).allowing(EDITOR);
        let members = Member::list(&names(), Some(&permissions));
        let roles: Vec<(&str, MemberRole)> = members
            .iter()
            .map(|member| (member.name(), member.role()))
            .collect();
        assert_eq!(
            roles,
            [
                ("e", MemberRole::Editor),
                ("o", MemberRole::Owner),
                ("v", MemberRole::Viewer)
            ]
        );
        assert_eq!(members[0].id(), EDITOR);
        assert_eq!(members[0].short_id(), "0000");
        let open = Member::list(&names(), None);
        assert!(
            open.iter()
                .all(|member| member.role() == MemberRole::Anyone)
        );
        assert_eq!(
            MemberRole::Anyone.word(),
            nene_docket_core::word::Word::Anyone
        );
    }
}
