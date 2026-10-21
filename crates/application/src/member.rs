//! 台帳に参加したアカウント 1 人（FR-009）。`keys/` の名前と、権限から決まる役と、入っているグループ。
//! 引き継がれた ID（ADR 0013）は、引き継いだ ID が同じ人なので出さない。

use alloc::string::String;
use alloc::vec::Vec;

use nene_docket_core::account_id::AccountId;
use nene_docket_core::permissions::Permissions;

use crate::member_role::MemberRole;

/// ID の先頭を出す桁数。
const ID_DIGITS: usize = 4;

/// ID・名前・役・グループ（名前の順）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    id: AccountId,
    name: String,
    role: MemberRole,
    groups: Vec<String>,
}

impl Member {
    /// `keys/` の名前と権限（無ければ誰でも編集）から、ID の順に並べる。引き継がれた ID は出さない。
    #[must_use]
    pub fn list(names: &[(AccountId, String)], permissions: Option<&Permissions>) -> Vec<Self> {
        let mut members: Vec<Self> = names
            .iter()
            .filter(|(id, _)| {
                permissions.is_none_or(|permissions| permissions.resolved(*id) == *id)
            })
            .map(|(id, name)| Self {
                id: *id,
                name: name.clone(),
                role: role_of(*id, permissions),
                groups: groups_of(*id, permissions),
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

    /// 入っているグループの名前（名前の順）。
    #[must_use]
    pub fn groups(&self) -> &[String] {
        &self.groups
    }
}

fn groups_of(id: AccountId, permissions: Option<&Permissions>) -> Vec<String> {
    permissions.map_or_else(Vec::new, |permissions| {
        permissions
            .groups()
            .iter()
            .filter(|(_, members)| members.contains(&id))
            .map(|(name, _)| name.clone())
            .collect()
    })
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
        let permissions = Permissions::owner_only(OWNER)
            .allowing(EDITOR)
            .grouped("qa", EDITOR)
            .grouped("dev", EDITOR);
        let members = Member::list(&names(), Some(&permissions));
        assert_eq!(members[0].groups(), ["dev", "qa"]);
        assert!(members[1].groups().is_empty());
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

    #[test]
    fn a_succeeded_id_is_left_out_and_the_successor_carries_its_role() {
        let permissions = Permissions::owner_only(OWNER)
            .allowing(EDITOR)
            .succeeding(EDITOR, VIEWER);
        let members = Member::list(&names(), Some(&permissions));
        let listed: Vec<AccountId> = members.iter().map(Member::id).collect();
        assert_eq!(listed, [EDITOR, OWNER]);
        assert_eq!(members[0].role(), MemberRole::Editor);
        assert_eq!(Member::list(&names(), None).len(), 3);
    }
}
