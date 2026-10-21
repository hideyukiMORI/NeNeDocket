//! 台帳の権限（FR-010・ADR 0010）。持ち主・編集者・グループ。持ち主が署名した `permissions.v1.txt` の中身。
//!
//! 持ち主はいつでも書ける。編集者と、どれかのグループに入っている人も書ける。他は読むだけ。
//! 引き継ぎ（ADR 0013）: 新しい ID が古い ID を引き継ぐと、古い ID は引き継いだ ID の役を持つ。
//! 一覧は昇順に保ち、同じ中身は同じ並びになる（署名する文が一意になる）。

use alloc::string::String;
use alloc::vec::Vec;

use crate::account_id::AccountId;

/// 持ち主・編集者・名前付きのグループ・引き継ぎ（新しい ID, 古い ID）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Permissions {
    owner: AccountId,
    editors: Vec<AccountId>,
    groups: Vec<(String, Vec<AccountId>)>,
    successions: Vec<(AccountId, AccountId)>,
}

impl Permissions {
    /// 持ち主だけが書ける。
    #[must_use]
    pub const fn owner_only(owner: AccountId) -> Self {
        Self {
            owner,
            editors: Vec::new(),
            groups: Vec::new(),
            successions: Vec::new(),
        }
    }

    /// 値から。一覧は並べ直し、重複を落とす。空のグループは落とす。
    #[must_use]
    pub fn new(
        owner: AccountId,
        editors: Vec<AccountId>,
        groups: Vec<(String, Vec<AccountId>)>,
    ) -> Self {
        let mut tidy = Self {
            owner,
            editors,
            groups,
            successions: Vec::new(),
        };
        tidy.tidy();
        tidy
    }

    /// 引き継ぎを置き換えた権限。古い ID ごとに 1 つ（後の指定が残る）。自分自身の引き継ぎは落とす。
    #[must_use]
    pub fn with_successions(mut self, successions: Vec<(AccountId, AccountId)>) -> Self {
        self.successions = successions;
        self.tidy();
        self
    }

    /// 引き継ぎ（新しい ID, 古い ID）。古い ID の順。
    #[must_use]
    pub fn successions(&self) -> &[(AccountId, AccountId)] {
        &self.successions
    }

    /// `new` が `old` を引き継いだ権限。`old` に前の引き継ぎがあれば置き換える。
    #[must_use]
    pub fn succeeding(mut self, new: AccountId, old: AccountId) -> Self {
        self.successions.retain(|(_, previous)| *previous != old);
        self.successions.push((new, old));
        self.tidy();
        self
    }

    /// 引き継ぎを辿った先の ID。引き継がれていなければそのまま。輪があっても引き継ぎの数で止まる。
    #[must_use]
    pub fn resolved(&self, id: AccountId) -> AccountId {
        let mut current = id;
        for _ in 0..self.successions.len() {
            match self.successions.iter().find(|(_, old)| *old == current) {
                Some((new, _)) => current = *new,
                None => break,
            }
        }
        current
    }

    /// 持ち主を `new` に替えた権限（ADR 0013 の 4）。前の持ち主は編集者に残る。編集者・グループ・引き継ぎはそのまま。
    #[must_use]
    pub fn handed_to(mut self, new: AccountId) -> Self {
        let previous = self.owner;
        self.owner = new;
        self.editors.push(previous);
        self.tidy();
        self
    }

    /// 持ち主。
    #[must_use]
    pub const fn owner(&self) -> AccountId {
        self.owner
    }

    /// 編集者（昇順）。持ち主は含まない。
    #[must_use]
    pub fn editors(&self) -> &[AccountId] {
        &self.editors
    }

    /// グループ（名前の順）と、その中のアカウント（昇順）。
    #[must_use]
    pub fn groups(&self) -> &[(String, Vec<AccountId>)] {
        &self.groups
    }

    /// `account` が書けるか。引き継がれた ID は、引き継いだ ID が書けるなら書ける。
    #[must_use]
    pub fn may_edit(&self, account: AccountId) -> bool {
        let resolved = self.resolved(account);
        self.may_edit_directly(account) || (resolved != account && self.may_edit_directly(resolved))
    }

    fn may_edit_directly(&self, account: AccountId) -> bool {
        account == self.owner
            || self.editors.contains(&account)
            || self
                .groups
                .iter()
                .any(|(_, members)| members.contains(&account))
    }

    /// 編集者を足した権限。
    #[must_use]
    pub fn allowing(mut self, account: AccountId) -> Self {
        self.editors.push(account);
        self.tidy();
        self
    }

    /// 編集者から外した権限。グループの中には残る。
    #[must_use]
    pub fn disallowing(mut self, account: AccountId) -> Self {
        self.editors.retain(|editor| *editor != account);
        self
    }

    /// グループに入れた権限。グループが無ければ作る。
    #[must_use]
    pub fn grouped(mut self, name: &str, account: AccountId) -> Self {
        match self.groups.iter_mut().find(|(group, _)| group == name) {
            Some((_, members)) => members.push(account),
            None => self.groups.push((String::from(name), alloc::vec![account])),
        }
        self.tidy();
        self
    }

    /// グループから外した権限。空になったグループは消える。
    #[must_use]
    pub fn ungrouped(mut self, name: &str, account: AccountId) -> Self {
        for (group, members) in &mut self.groups {
            if group == name {
                members.retain(|member| *member != account);
            }
        }
        self.tidy();
        self
    }

    fn tidy(&mut self) {
        self.editors.retain(|editor| *editor != self.owner);
        self.editors.sort_unstable();
        self.editors.dedup();
        for (_, members) in &mut self.groups {
            members.sort_unstable();
            members.dedup();
        }
        self.groups.retain(|(_, members)| !members.is_empty());
        self.successions.retain(|(new, old)| new != old);
        self.successions.reverse();
        self.successions.sort_by_key(|(_, old)| *old);
        self.successions.dedup_by_key(|(_, old)| *old);
        self.groups.sort_by(|(left, _), (right, _)| left.cmp(right));
        self.groups.dedup_by(|later, earlier| {
            if later.0 == earlier.0 {
                earlier.1.append(&mut later.1);
                earlier.1.sort_unstable();
                earlier.1.dedup();
                true
            } else {
                false
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OWNER: AccountId = AccountId::new(1);
    const A: AccountId = AccountId::new(2);
    const B: AccountId = AccountId::new(3);
    const C: AccountId = AccountId::new(4);

    #[test]
    fn the_owner_editors_and_group_members_may_edit_and_others_may_not() {
        let permissions = Permissions::owner_only(OWNER).allowing(A).grouped("qa", B);
        assert!(permissions.may_edit(OWNER));
        assert!(permissions.may_edit(A));
        assert!(permissions.may_edit(B));
        assert!(!permissions.may_edit(C));
        assert!(!permissions.disallowing(A).may_edit(A));
        let left = Permissions::owner_only(OWNER)
            .grouped("qa", B)
            .ungrouped("qa", B);
        assert!(!left.may_edit(B));
        assert!(left.groups().is_empty());
    }

    #[test]
    fn a_succeeded_id_inherits_the_successors_role_and_chains_are_followed() {
        let old = AccountId::new(10);
        let middle = AccountId::new(11);
        let new = AccountId::new(12);
        let permissions = Permissions::owner_only(OWNER)
            .allowing(new)
            .succeeding(middle, old)
            .succeeding(new, middle);
        assert_eq!(permissions.resolved(old), new);
        assert_eq!(permissions.resolved(new), new);
        assert_eq!(permissions.resolved(C), C);
        assert!(permissions.may_edit(old));
        assert!(permissions.may_edit(middle));
        assert!(!permissions.may_edit(C));
        assert_eq!(permissions.successions(), [(middle, old), (new, middle)]);
    }

    #[test]
    fn successions_keep_one_per_old_id_drop_self_successions_and_survive_cycles() {
        let old = AccountId::new(10);
        let permissions = Permissions::owner_only(OWNER)
            .succeeding(A, old)
            .succeeding(C, old)
            .succeeding(C, C);
        assert_eq!(permissions.resolved(old), C);
        assert_eq!(permissions.successions(), [(C, old)]);
        assert_eq!(
            Permissions::owner_only(OWNER).with_successions(alloc::vec![(A, B), (C, B)]),
            Permissions::owner_only(OWNER).succeeding(C, B)
        );
        let cycle = Permissions::owner_only(OWNER)
            .succeeding(A, B)
            .succeeding(B, A);
        assert!(!cycle.may_edit(A));
        assert!(cycle.resolved(A) == A || cycle.resolved(A) == B);
    }

    #[test]
    fn handing_over_keeps_the_lists_and_leaves_the_previous_owner_as_an_editor() {
        let handed = Permissions::owner_only(OWNER)
            .allowing(A)
            .grouped("qa", B)
            .succeeding(C, B)
            .handed_to(A);
        assert_eq!(handed.owner(), A);
        assert_eq!(handed.editors(), [OWNER]);
        assert_eq!(handed.groups(), [(String::from("qa"), alloc::vec![B])]);
        assert_eq!(handed.successions(), [(C, B)]);
        assert_eq!(handed.clone().handed_to(A), handed);
    }

    #[test]
    fn lists_are_sorted_deduplicated_and_free_of_the_owner() {
        let permissions = Permissions::new(
            OWNER,
            alloc::vec![B, A, A, OWNER],
            alloc::vec![
                (String::from("z"), alloc::vec![C, B]),
                (String::from("a"), alloc::vec![A]),
                (String::from("z"), alloc::vec![B]),
                (String::from("empty"), Vec::new()),
            ],
        );
        assert_eq!(permissions.owner(), OWNER);
        assert_eq!(permissions.editors(), [A, B]);
        assert_eq!(
            permissions.groups(),
            [
                (String::from("a"), alloc::vec![A]),
                (String::from("z"), alloc::vec![B, C])
            ]
        );
        assert_eq!(permissions.clone().allowing(A).grouped("a", A), permissions);
    }
}
